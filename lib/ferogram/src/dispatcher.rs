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

use crate::{Result, Router};

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
        for router in self.routers.lock().await.iter_mut() {
            match router.handle_update(&client, &update).await {
                Ok(true) => return Ok(()),
                Ok(false) => continue,
                Err(e) => return Err(format!("Error handling update on router: {:?}", e).into()),
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
