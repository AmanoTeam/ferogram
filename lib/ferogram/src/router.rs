// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Router module.

use async_recursion::async_recursion;
use grammers_client::Update;

use crate::{di::Injector, Handler, Result};

/// A router.
///
/// Sends updates to the handlers.
#[derive(Clone, Default)]
pub struct Router {
    /// The handlers.
    pub(crate) handlers: Vec<Handler>,
    /// The routers.
    pub(crate) routers: Vec<Router>,
}

impl Router {
    /// Attachs a new handler.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let router = unimplemented!();
    /// let router = router.handler(handler::then(|| async { Ok(()) }));
    /// # }
    /// ```
    pub fn handler(mut self, handler: Handler) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Attachs a new router.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let router = unimplemented!();
    /// let router = router.router(|router| {
    ///     router
    /// });
    /// # }
    /// ```
    pub fn router<R: FnOnce(Router) -> Router + 'static>(mut self, router: R) -> Self {
        let router = router(Self::default());
        self.handlers.extend(router.handlers);
        self
    }

    /// Handle the update sent by Telegram.
    ///
    /// Returns `Ok(())` if the update was handled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// use ferogram::di::Injector;
    ///
    /// # let router = unimplemented!();
    /// let mut injector = Injector::default();
    /// let success = router.handle_update(&client, &update, &mut injector).await?;
    /// # }
    /// ```
    #[async_recursion]
    pub(crate) async fn handle_update(
        &mut self,
        client: &grammers_client::Client,
        update: &Update,
        injector: &mut Injector,
    ) -> Result<bool> {
        for handler in self.handlers.iter_mut() {
            let flow = handler.check(&client, &update).await;

            if flow.is_continue() {
                if let Some(endpoint) = handler.endpoint.as_mut() {
                    let mut handler_injector = flow.injector;
                    injector.extend(&mut handler_injector);

                    match update.clone() {
                        Update::NewMessage(message) | Update::MessageEdited(message) => {
                            injector.insert(message)
                        }
                        Update::MessageDeleted(message_deletion) => {
                            injector.insert(message_deletion)
                        }
                        Update::CallbackQuery(query) => injector.insert(query),
                        Update::InlineQuery(query) => injector.insert(query),
                        Update::InlineSend(inline_send) => injector.insert(inline_send),
                        Update::Raw(raw) => injector.insert(raw),
                        _ => {}
                    }

                    match endpoint.handle(injector).await {
                        Ok(()) => return Ok(true),
                        Err(e) => {
                            if let Some(err_filter) = handler.err_handler.as_mut() {
                                let flow = err_filter.run(client.clone(), update.clone(), e).await;

                                if flow.is_continue() {
                                    let mut flow_injector = flow.injector;
                                    injector.extend(&mut flow_injector);

                                    return endpoint.handle(injector).await.map(|_| true);
                                }

                                return Ok(true);
                            }

                            return Err(e);
                        }
                    }
                }
            }
        }

        for router in self.routers.iter_mut() {
            match router.handle_update(client, update, injector).await {
                Ok(false) => continue,
                r @ Ok(true) => return r,
                Err(e) => return Err(e),
            }
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handler;

    #[test]
    fn router() {
        let filter = |_, _| async { true };
        let endpoint = || async { Ok(()) };

        let router = Router::default()
            .handler(handler::then(|| async { Ok(()) }))
            .handler(handler::new_message(|_, _| async { true }))
            .handler(handler::new_update(filter).then(endpoint))
            .handler(handler::then(|_update: Update| async { Ok(()) }));

        assert_eq!(router.handlers.len(), 3);
    }
}
