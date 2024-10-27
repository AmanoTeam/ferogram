// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_recursion::async_recursion;
use grammers_client::Update;

use crate::{di, Handler, Result};

/// Dispatcher's router
#[derive(Clone, Default)]
pub struct Router {
    handlers: Vec<Handler>,
    routers: Vec<Router>,
}

impl Router {
    /// Attach a new handler.
    pub fn handler(mut self, handler: Handler) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Attach a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(mut self, router: R) -> Self {
        let router = router(Self::default());
        self.handlers.extend(router.handlers);
        self
    }

    /// Handle the update sent by Telegram.
    #[async_recursion]
    pub(crate) async fn handle_update(
        &mut self,
        client: &grammers_client::Client,
        update: &Update,
    ) -> Result<bool> {
        for handler in self.handlers.iter_mut() {
            if handler.check(&client, &update).await {
                if let Some(endpoint) = handler.endpoint.as_mut() {
                    let mut injector = di::Injector::new();

                    injector.insert(client.clone());
                    injector.insert(update.clone());

                    endpoint.handle(injector).await?;
                }

                return Ok(true);
            }
        }

        for router in self.routers.iter_mut() {
            match router.handle_update(client, update).await {
                Ok(true) => return Ok(true),
                Ok(false) => continue,
                Err(e) => return Err(format!("Error handling update: {:?}", e).into()),
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
            .handler(handler::new_update(filter).then(endpoint));

        assert_eq!(router.handlers.len(), 3);
    }
}
