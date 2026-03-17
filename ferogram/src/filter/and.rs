// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use futures_core::future::BoxFuture;
use grammers::{Client, update::Update};

use super::{AsyncMarker, Filter, Flow, IntoFilter};

/// `And` filter, which contains two filters and only allows the execution of
/// the [`crate::Endpoint`] if both pass.
pub struct AndFilter {
    pub(crate) first: Box<dyn Filter>,
    pub(crate) second: Box<dyn Filter>,
}

impl Filter for AndFilter {
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move {
            let mut first_flow = self.first.run(&client, &update).await;

            if first_flow.is_proceed() {
                let second_flow = self.second.run(&client, &update).await;

                if second_flow.is_proceed() {
                    let first_injector = &mut first_flow.injector;
                    let second_injector = second_flow.injector;
                    first_injector.extend(second_injector);

                    return first_flow;
                }
            }

            super::stop()
        })
    }
}

impl IntoFilter<AsyncMarker> for AndFilter {
    type Filter = AndFilter;

    fn into_filter(self) -> Self::Filter {
        self
    }
}
