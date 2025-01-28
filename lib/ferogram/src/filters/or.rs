// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_trait::async_trait;
use grammers_client::{Client, Update};

use crate::{flow, Filter, Flow};

#[derive(Clone)]
pub struct Or {
    pub(crate) first: Box<dyn Filter>,
    pub(crate) other: Box<dyn Filter>,
}

#[async_trait]
impl Filter for Or {
    async fn check(&mut self, client: Client, update: Update) -> Flow {
        let first_flow = self.first.check(client.clone(), update.clone()).await;

        if first_flow.is_continue() {
            first_flow
        } else {
            let other_flow = self.other.check(client, update).await;

            if other_flow.is_continue() {
                other_flow
            } else {
                flow::break_now()
            }
        }
    }
}
