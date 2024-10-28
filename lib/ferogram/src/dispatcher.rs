// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use grammers_client::{Client, Update};
use tokio::sync::Mutex;

use crate::{di, Result, Router};

/// Dispatcher
#[derive(Clone, Default)]
pub struct Dispatcher {
    routers: Arc<Mutex<Vec<Router>>>,
}

impl Dispatcher {
    /// Attach a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(self, router: R) -> Self {
        let router = router(Router::default());
        self.routers.try_lock().unwrap().push(router);

        self
    }

    /// Handle the update sent by Telegram.
    pub(crate) async fn handle_update(&mut self, client: Client, update: Update) -> Result<()> {
        let mut main_injector = None;

        for router in self.routers.lock().await.iter_mut() {
            if main_injector.is_none() {
                let mut injector = di::Injector::new();
                injector.insert(client.clone());
                injector.insert(update.clone());

                main_injector = Some(injector);
            }

            match router
                .handle_update(&client, &update, main_injector.unwrap())
                .await
            {
                Ok(None) => return Ok(()),
                Ok(injector) => main_injector = injector,
                Err(e) => return Err(format!("Error handling update: {:?}", e).into()),
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
