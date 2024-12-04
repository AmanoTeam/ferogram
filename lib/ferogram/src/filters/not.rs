// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_trait::async_trait;
use grammers_client::{Client, Update};

use crate::{Filter, Flow};

#[derive(Clone)]
pub struct Not {
    pub(crate) filter: Box<dyn Filter>,
}

#[async_trait]
impl Filter for Not {
    async fn check(&mut self, client: Client, update: Update) -> Flow {
        self.filter
            .check(client.clone(), update.clone())
            .await
            .is_break()
            .into()
    }
}
