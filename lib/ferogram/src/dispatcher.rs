// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Dispatcher module.

use grammers_client::{types::Chat, Client, Update};
use tokio::sync::broadcast::Sender;

use crate::{di, Context, Plugin, Result, Router};

/// A dispatcher.
///
/// Sends the updates to the routers and plugins.
#[derive(Clone)]
pub struct Dispatcher {
    /// The routers.
    routers: Vec<Router>,
    /// The plugins.
    plugins: Vec<Plugin>,
    /// The main injector.
    injector: di::Injector,
    /// The update sender.
    upd_sender: Sender<Update>,

    /// Whether allow the client to handle updates from itself.
    allow_from_self: bool,
}

impl Dispatcher {
    /// Attachs a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(mut self, router: R) -> Self {
        let router = router(Router::default());
        self.routers.push(router);

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
    pub fn plugin(mut self, plugin: Plugin) -> Self {
        self.plugins.push(plugin);

        self
    }

    /// Handle the update sent by Telegram.
    pub(crate) async fn handle_update(&mut self, client: &Client, update: &Update) -> Result<()> {
        let mut injector = di::Injector::default();

        let upd_receiver = self.upd_sender.subscribe();
        let context = Context::new(client, update, upd_receiver);
        injector.insert(context);

        self.upd_sender
            .send(update.clone())
            .expect("Failed to send update");

        injector.insert(client.clone());
        injector.insert(update.clone());
        injector.extend(&mut self.injector.clone());

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

        for router in self.routers.iter_mut() {
            match router.handle_update(client, update, &mut injector).await {
                Ok(false) => continue,
                Ok(true) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        for plugin in self.plugins.iter_mut() {
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

impl Default for Dispatcher {
    fn default() -> Self {
        let (upd_sender, _) = tokio::sync::broadcast::channel(10);

        Self {
            routers: Vec::new(),
            plugins: Vec::new(),
            injector: di::Injector::default(),
            upd_sender,

            allow_from_self: false,
        }
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
