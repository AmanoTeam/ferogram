// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Router module.

use async_recursion::async_recursion;
use grammers_client::Update;

use crate::{di::Injector, filter::Command, middleware::MiddlewareStack, Handler, Result};

/// A router.
///
/// Sends updates to the handlers.
#[derive(Clone, Default)]
pub struct Router {
    /// The handlers.
    pub(crate) handlers: Vec<Handler>,
    /// The routers.
    pub(crate) routers: Vec<Router>,
    /// The middleware stack.
    pub(crate) middlewares: MiddlewareStack,
}

impl Router {
    /// Attachs a new handler.
    ///
    /// # Example
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
    /// # Example
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

    /// Attachs a middleware stack.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let router = unimplemented!();
    /// let router = router.middlewares(|middlewares| {
    ///     middlewares
    ///         .before(|_, _, _| async { Ok(flow::continue_now()) })
    ///         .after(|_, _, _| async { Ok(flow::continue_now()) })
    /// });
    /// # }
    /// ```
    pub fn middlewares<M: FnOnce(MiddlewareStack) -> MiddlewareStack>(
        mut self,
        middlewares: M,
    ) -> Self {
        self.middlewares = middlewares(self.middlewares);
        self
    }

    /// Returns the commands from the handlers.
    pub(crate) fn get_commands(&self) -> Vec<Command> {
        let mut commands = Vec::new();

        commands.extend(
            self.handlers
                .iter()
                .filter_map(|handler| handler.command.clone()),
        );
        commands.extend(self.routers.iter().flat_map(|router| router.get_commands()));

        commands
    }

    /// Handle the update sent by Telegram.
    ///
    /// Returns `Ok(())` if the update was handled.
    ///
    /// # Example
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
        middlewares: MiddlewareStack,
    ) -> Result<bool> {
        let mut middlewares = middlewares.extend(self.middlewares.clone());

        for handler in self.handlers.iter_mut() {
            let mut middleware_flow = middlewares.handle_before(client, update, injector).await;
            if middleware_flow.is_continue() {
                let mut flow = handler.check(client, update).await;
                flow.injector.extend(&mut middleware_flow.injector);

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
                            Ok(()) => {
                                return {
                                    middlewares.handle_after(client, update, injector).await;

                                    Ok(true)
                                }
                            }
                            Err(e) => {
                                if let Some(err_filter) = handler.err_handler.as_mut() {
                                    let flow =
                                        err_filter.run(client.clone(), update.clone(), e).await;

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
        }

        for router in self.routers.iter_mut() {
            match router
                .handle_update(client, update, injector, middlewares.clone())
                .await
            {
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
    use async_trait::async_trait;
    use grammers_client::Client;

    use super::*;
    use crate::{flow, handler, Flow, Middleware};

    #[test]
    fn router() {
        let filter = |_, _| async { true };
        let endpoint = || async { Ok(()) };

        let router = Router::default()
            .handler(handler::then(|| async { Ok(()) }))
            .handler(handler::new_message(|_, _| async { true }))
            .handler(handler::new_update(filter).then(endpoint))
            .handler(handler::then(|_update: Update| async { Ok(()) }));

        assert_eq!(router.handlers.len(), 4);
    }

    #[derive(Clone)]
    struct TestMiddleware;

    #[async_trait]
    impl Middleware for TestMiddleware {
        async fn handle(
            &mut self,
            _client: &Client,
            _update: &Update,
            _injector: &mut Injector,
        ) -> Flow {
            flow::break_now()
        }
    }

    #[test]
    fn test_middlewares() {
        let router = Router {
            handlers: Vec::new(),
            routers: Vec::new(),
            middlewares: MiddlewareStack::new(),
        };

        let updated_router = router
            .middlewares(|middlewares| middlewares.before(TestMiddleware).after(TestMiddleware));

        assert_eq!(updated_router.middlewares.before.len(), 1);
        assert_eq!(updated_router.middlewares.after.len(), 1);
    }
}
