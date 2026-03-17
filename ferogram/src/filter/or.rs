// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use futures_core::future::BoxFuture;
use grammers::{Client, update::Update};

use super::{AsyncMarker, Filter, Flow, IntoFilter};

/// `Or` filter, which contains two filters and allows the execution of the
/// [`crate::di::Endpoint`] if any of them pass.
pub struct OrFilter {
    pub(crate) first: Box<dyn Filter>,
    pub(crate) other: Box<dyn Filter>,
}

impl Filter for OrFilter {
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move {
            let first_flow = self.first.run(&client, &update).await;

            if first_flow.is_proceed() {
                first_flow
            } else {
                let other_flow = self.other.run(&client, &update).await;

                if other_flow.is_proceed() {
                    other_flow
                } else {
                    super::stop()
                }
            }
        })
    }
}

impl IntoFilter<AsyncMarker> for OrFilter {
    type Filter = OrFilter;

    fn into_filter(self) -> Self::Filter {
        self
    }
}
