// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dispatcher module.

use std::sync::Arc;

use grammers_client::{types::Chat, Client, Update};
use tokio::sync::Mutex;

use crate::{di, Context, Plugin, Result, Router};

/// A dispatcher.
///
/// Sends the updates to the routers and plugins.
#[derive(Default)]
pub struct Dispatcher {
    routers: Arc<Mutex<Vec<Router>>>,
    plugins: Arc<Mutex<Vec<Plugin>>>,
    injector: di::Injector,
    last_context: Arc<Mutex<Option<Context>>>,

    allow_from_self: bool,
}

impl Dispatcher {
    /// Attachs a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(self, router: R) -> Self {
        let router = router(Router::default());
        // `router()` only is executed on startup, so `routers` never will be locked.
        self.routers.try_lock().unwrap().push(router);

        self
    }

    /// Attachs a injector.
    pub fn resources<D: FnOnce(di::Injector) -> di::Injector>(mut self, injector: D) -> Self {
        let mut injector = injector(di::Injector::default());
        self.injector.extend(&mut injector);

        self
    }

    /// Attachs a injector.
    ///
    /// Same as `resources`.
    pub fn dependencies<D: FnOnce(di::Injector) -> di::Injector>(self, injector: D) -> Self {
        self.resources(injector)
    }

    /// Allows the client to handle updates from itself.
    pub fn allow_from_self(mut self) -> Self {
        self.allow_from_self = true;
        self
    }

    /// Attachs a new plugin.
    pub fn plugin(self, plugin: Plugin) -> Self {
        // `plugin()` only is executed on startup, so `plugins` never will be locked.
        self.plugins.try_lock().unwrap().push(plugin);

        self
    }

    /// Handle the update sent by Telegram.
    pub(crate) async fn handle_update(&self, client: &Client, update: &Update) -> Result<()> {
        let mut routers = self.routers.lock().await;
        let mut plugins = self.plugins.lock().await;

        let mut injector = di::Injector::default();
        injector.insert(client.clone());
        injector.insert(update.clone());
        injector.extend(&mut self.injector.clone());

        let mut last_context = self.last_context.lock().await;

        if let Some(context) = last_context.as_ref() {
            if context.is_waiting_for_update() {
                return Ok(());
            }
        }

        let context = Context::new(&client, &update);
        last_context.replace(context.clone());
        injector.insert(context);

        if !self.allow_from_self {
            match update {
                Update::NewMessage(message) | Update::MessageEdited(message) => {
                    if let Some(Chat::User(user)) = message.sender() {
                        if user.is_self() {
                            return Ok(());
                        }
                    }
                }
                Update::CallbackQuery(query) => {
                    if let Chat::User(user) = query.sender() {
                        if user.is_self() {
                            return Ok(());
                        }
                    }
                }
                Update::InlineQuery(query) => {
                    let user = query.sender();

                    if user.is_self() {
                        return Ok(());
                    }
                }
                Update::InlineSend(inline_send) => {
                    let user = inline_send.sender();

                    if user.is_self() {
                        return Ok(());
                    }
                }
                _ => {}
            };
        }

        for router in routers.iter_mut() {
            match router.handle_update(client, update, &mut injector).await {
                Ok(false) => continue,
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        for plugin in plugins.iter_mut() {
            match plugin
                .router
                .handle_update(client, update, &mut injector)
                .await
            {
                Ok(false) => continue,
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler;

    #[test]
    fn test_dispatcher() {
        Dispatcher::default()
            .router(|router| router)
            .router(|router| {
                router.handler(handler::then(|_: Client, _: Update| async { Ok(()) }))
            });
    }
}
