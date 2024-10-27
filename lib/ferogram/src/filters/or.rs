// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use async_trait::async_trait;
use grammers_client::{Client, Update};

use crate::{Filter, Flow};

pub struct Or {
    pub(crate) first: Arc<dyn Filter>,
    pub(crate) other: Arc<dyn Filter>,
}

#[async_trait]
impl Filter for Or {
    async fn check(&self, client: Client, update: Update) -> Flow {
        (self
            .first
            .check(client.clone(), update.clone())
            .await
            .is_continue()
            || self.other.check(client, update).await.is_continue())
        .into()
    }
}
