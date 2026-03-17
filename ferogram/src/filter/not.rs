// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use futures_core::future::BoxFuture;
use grammers::{Client, update::Update};

use super::{AsyncMarker, Filter, Flow, IntoFilter};

/// `Not` filter, which contains one filter and allows the execution of the
/// [`crate::Endpoint`] if it don't pass.
pub struct NotFilter {
    pub(crate) filter: Box<dyn Filter>,
}

impl Filter for NotFilter {
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move {
            let flow = self.filter.run(&client, &update).await;

            if flow.is_stop() { flow } else { super::stop() }
        })
    }
}

impl IntoFilter<AsyncMarker> for NotFilter {
    type Filter = NotFilter;

    fn into_filter(self) -> Self::Filter {
        self
    }
}
