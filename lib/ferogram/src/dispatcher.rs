// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use grammers_client::{Client, Update};

use crate::{Result, Router};

/// Dispatcher
#[derive(Clone, Default)]
pub struct Dispatcher {
    routers: Vec<Router>,
}

impl Dispatcher {
    /// Attach a new router.
    pub fn router<R: FnOnce(Router) -> Router + 'static>(mut self, router: R) -> Self {
        let router = router(Router::default());
        self.routers.push(router);
        self
    }

    /// Handle the update sent by Telegram.
    pub(crate) async fn handle_update(&self, client: Client, update: Update) -> Result<()> {
        for router in self.routers.iter() {
            match router.handle_update(client.clone(), update.clone()).await {
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
        let dispatcher = Dispatcher::default()
            .router(|router| router)
            .router(|router| router.handler(handler::then(|_, _| async { Ok(()) })));

        assert_eq!(dispatcher.routers.len(), 2);
    }
}
