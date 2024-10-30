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

use crate::{flow, FilterHandler, Flow};

pub struct Or {
    pub(crate) first: Arc<dyn FilterHandler>,
    pub(crate) other: Arc<dyn FilterHandler>,
}

#[async_trait]
impl FilterHandler for Or {
    async fn check(&self, client: Client, update: Update) -> Flow {
        let first_flow = self.first.check(client.clone(), update.clone()).await;
        let other_flow = self.other.check(client, update).await;

        if first_flow.is_continue() {
            first_flow
        } else if other_flow.is_continue() {
            other_flow
        } else {
            flow::break_now()
        }
    }
}
