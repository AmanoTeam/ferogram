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

use crate::{di, Result, Router};

/// Dispatcher
#[derive(Default)]
pub struct Dispatcher {
    routers: Arc<Mutex<Vec<Router>>>,
    injector: di::Injector,

    allow_from_self: bool,
}

impl Dispatcher {
    /// Attach a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(self, router: R) -> Self {
        let router = router(Router::default());
        // `router()` only is executed on startup, so `routers` never will be locked.
        self.routers.try_lock().unwrap().push(router);

        self
    }

    /// Attach a injector.
    pub fn resources<D: FnOnce(di::Injector) -> di::Injector>(mut self, injector: D) -> Self {
        let mut injector = injector(di::Injector::default());
        self.injector.extend(&mut injector);

        self
    }

    /// Attach a injector.
    ///
    /// Same as `resources`.
    pub fn dependencies<D: FnOnce(di::Injector) -> di::Injector>(self, injector: D) -> Self {
        self.resources(injector)
    }

    /// Allow the client to handle updates from itself.
    pub fn allow_from_self(mut self) -> Self {
        self.allow_from_self = true;
        self
    }

    /// Handle the update sent by Telegram.
    pub(crate) async fn handle_update(&self, client: &Client, update: &Update) -> Result<()> {
        let mut routers = self.routers.lock().await;

        let mut injector = di::Injector::default();
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

        for router in routers.iter_mut() {
            match router.handle_update(client, update, &mut injector).await {
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
