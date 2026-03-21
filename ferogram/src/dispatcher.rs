// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! A listener that dispatches updates to handlers.

use std::sync::{Arc, LazyLock};

use grammers::{Client, SenderPool, client::UpdatesConfiguration, peer::Peer, update::Update};
use tokio::{
    sync::{
        Mutex, Notify,
        broadcast::{self, Sender},
    },
    task::{JoinHandle, JoinSet},
};

use crate::{Context, Handler, Injector, wait_for_ctrl_c};

/// A notification sent to stop the dispatcher.
pub(super) static STOP_DISPATCHER: LazyLock<Arc<Notify>> =
    LazyLock::new(|| Arc::new(Notify::new()));

/// A notification sent when the dispatcher is fully stopped.
pub(super) static DISPATCHER_STOPPED: LazyLock<Arc<Notify>> =
    LazyLock::new(|| Arc::new(Notify::new()));

/// A update dispatcher.
///
/// It receives the updates from Telegram by wrapping [`Client`] and
/// dispatches it to the their right handler.
pub struct Dispatcher {
    /// Routes to be dispatched based on its filters.
    handlers: Mutex<Vec<Handler>>,
    /// Update sender to contexts.
    update_tx: Sender<Update>,
    /// Whether allow the client to handle updates from itself.
    allow_from_self: bool,
}

impl Dispatcher {
    /// Create a new builder.
    pub fn builder() -> DispatcherBuilder {
        DispatcherBuilder::default()
    }

    /// Keep the connection open, but don't listen to nor dispatch any update.
    ///
    /// Note that this calls [`crate::wait_for_ctrl_c`] to disconnect the client
    /// after `Ctrl+C` signal is received, so you might need to spawn it in a task.
    pub async fn keep_alive(self, pool: SenderPool) {
        let SenderPool { runner, handle, .. } = pool;
        let pool_task = tokio::task::spawn(runner.run());

        wait_for_ctrl_c().await;

        handle.quit();
        let _ = pool_task.await;
    }

    /// Listen to Telegram's updates and dispatches them to their rightful handlers.
    ///
    /// It runs on a background task, so you may need to use [`crate::idle`] or await
    /// the returned [`JoinHandle`].
    pub fn run(
        self,
        pool: SenderPool,
        client: Client,
        configuration: UpdatesConfiguration,
    ) -> JoinHandle<impl Send> {
        let this = Arc::new(self);

        let SenderPool {
            runner,
            handle,
            updates,
        } = pool;
        let pool_task = tokio::task::spawn(runner.run());

        tracing::info!("The pool is ready to receive updates!");

        tokio::task::spawn(async move {
            let mut handler_tasks = JoinSet::new();
            let mut updates = client.stream_updates(updates, configuration).await;

            loop {
                tokio::select! {
                    _ = STOP_DISPATCHER.notified() => break,
                    update = updates.next() => {
                        let update = update.unwrap();

                        let dp = Arc::clone(&this);
                        let client = client.clone();

                        let mut injector = Injector::default();
                        injector.push(client.clone());
                        injector.push(update.clone());

                        let update_rx = dp.update_tx.subscribe();
                        let context = Context::new(client.clone(), update.clone(), update_rx);
                        injector.push(context);

                        dp.update_tx.send(update.clone()).expect("Failed to send update to open contexts");

                        handler_tasks.spawn(async move {
                            if !dp.allow_from_self {
                                match update {
                                    Update::NewMessage(ref message)
                                    | Update::MessageEdited(ref message) => {
                                        if let Some(Peer::User(user)) = message.peer()
                                            && user.is_self()
                                        {
                                            return;
                                        }
                                    }
                                    Update::CallbackQuery(ref query) => {
                                        if let Some(Peer::User(user)) = query.sender()
                                            && user.is_self()
                                        {
                                            return;
                                        }
                                    }
                                    Update::InlineQuery(ref query) => {
                                        if let Some(user) = query.sender()
                                            && user.is_self()
                                        {
                                            return;
                                        }
                                    }
                                    Update::InlineSend(ref query) => {
                                        if let Some(user) = query.sender()
                                            && user.is_self()
                                        {
                                            return;
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            let mut handlers = dp.handlers.lock().await;
                            for handler in handlers.iter_mut() {
                                match handler.run(&client, &update, injector.clone()).await {
                                    Ok(false) => continue,
                                    Ok(true) => return,
                                    Err(e) => {
                                        tracing::error!("An error ocurred while executing a handler: {e}")
                                    }
                                }
                            }
                        });
                    }
                }
            }

            tracing::info!("Saving session...");
            updates.sync_update_state().await;

            tracing::info!("Exiting...");
            handle.quit();
            let _ = pool_task.await;

            tracing::info!("Waiting for any slow handlers to finish...");
            while handler_tasks.try_join_next().is_some() {}

            DISPATCHER_STOPPED.notify_waiters();
        })
    }
}

/// Just a dispatcher builder.
#[derive(Default)]
pub struct DispatcherBuilder {
    /// Routes to be dispatched based on its filters.
    handlers: Vec<Handler>,
    /// Whether allow the client to handle updates from itself.
    allow_from_self: bool,
}

impl DispatcherBuilder {
    /// Terminate the building process.
    pub fn build(self) -> Dispatcher {
        let (update_tx, _) = broadcast::channel(10);

        Dispatcher {
            handlers: Mutex::new(self.handlers),
            update_tx,
            allow_from_self: self.allow_from_self,
        }
    }

    /// Add a handler to the dispatcher.
    ///
    /// It has priority over [`Router`]'s, which is why it will be run before
    /// any router.
    pub fn add_handler(mut self, handler: Handler) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Allow the client to handle updates from itself.
    ///
    /// By default, the client will not handle updates from itself.
    pub fn allow_from_self(mut self) -> Self {
        self.allow_from_self = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{filter, handler};

    use super::*;

    #[test]
    fn test_dispatcher() {
        Dispatcher::builder()
            .allow_from_self()
            .add_handler(handler::new_message(filter::always).then(|| async { Ok(()) }))
            .build();
    }
}
