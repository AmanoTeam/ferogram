// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use futures::{future::BoxFuture, Future};
use grammers_client::{Client, Update};

use crate::Result;

/// Update endpoint.
pub trait Endpoint: Send + Sync + 'static {
    /// Handle the update.
    fn handle(&self, client: Client, update: Update) -> BoxFuture<'static, Result<()>>;
}

impl<T: Clone, F> Endpoint for T
where
    T: Fn(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = Result<()>> + Send + Sync + 'static,
{
    fn handle(&self, client: Client, update: Update) -> BoxFuture<'static, Result<()>> {
        Box::pin(self(client, update))
    }
}
