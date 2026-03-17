// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Async and sync markers to make [`Filter`] accept async and sync functions.

use futures_core::future::BoxFuture;
use grammers::{Client, update::Update};

use super::{Filter, Flow};

/// Async dummy marker.
pub struct AsyncMarker;
/// Sync dummy marker.
pub struct SyncMarker;

/// Adapter trait to converting a marker filter to a proper [`Filter`].
pub trait IntoFilter<Marker> {
    type Filter: Filter;

    fn into_filter(self) -> Self::Filter;
}

impl<T: Clone, F, O> IntoFilter<AsyncMarker> for T
where
    T: FnMut(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send,
    O: Into<Flow>,
{
    type Filter = T;

    fn into_filter(self) -> Self::Filter {
        self
    }
}

impl<T: Clone, O> IntoFilter<SyncMarker> for T
where
    T: FnMut(Client, Update) -> O + Send + Sync + 'static,
    O: Into<Flow>,
{
    type Filter = SyncFilter<T>;

    fn into_filter(self) -> Self::Filter {
        SyncFilter(self)
    }
}

/// Wrapper about a sync filter.
#[derive(Clone)]
pub struct SyncFilter<T>(pub T);

impl<T, O> Filter for SyncFilter<T>
where
    T: FnMut(Client, Update) -> O + Send + Sync + 'static,
    O: Into<Flow>,
{
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move { (self.0)(client, update).into() })
    }
}
