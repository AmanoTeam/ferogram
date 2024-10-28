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

use crate::{flow, Filter, Flow};

pub struct And {
    pub(crate) first: Arc<dyn Filter>,
    pub(crate) second: Arc<dyn Filter>,
}

#[async_trait]
impl Filter for And {
    async fn check(&self, client: Client, update: Update) -> Flow {
        let first_flow = self.first.check(client.clone(), update.clone()).await;
        let second_flow = self.second.check(client, update).await;

        let mut first_injector = first_flow.injector.lock().await;
        let mut second_injector = second_flow.injector.lock().await;

        second_injector.extend(&mut first_injector);
        drop(second_injector);

        if first_flow.is_continue() && second_flow.is_continue() {
            second_flow
        } else {
            flow::break_now()
        }
    }
}
