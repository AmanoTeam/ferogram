// Copyright 2024 - Andriel Ferreira
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
pub struct And {
    pub(crate) first: Box<dyn Filter>,
    pub(crate) second: Box<dyn Filter>,
}

#[async_trait]
impl Filter for And {
    async fn check(&mut self, client: Client, update: Update) -> Flow {
        let mut first_flow = self.first.check(client.clone(), update.clone()).await;

        if first_flow.is_continue() {
            let second_flow = self.second.check(client, update).await;

            if second_flow.is_continue() {
                let first_injector = &mut first_flow.injector;
                let mut second_injector = second_flow.injector;

                first_injector.extend(&mut second_injector);

                return first_flow;
            }
        }

        flow::break_now()
    }
}
