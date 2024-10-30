// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Filters module.

use std::sync::Arc;

use async_trait::async_trait;
use futures::Future;
use grammers_client::{Client, Update};

pub use crate::filters::*;
use crate::{flow, Flow};

/// Update filter.
///
/// Checked at each update to know if the update should be handled.
#[async_trait]
pub trait FilterHandler: Send + Sync + 'static {
    /// Check if the update should be handled.
    async fn check(&self, client: Client, update: Update) -> Flow;

    /// Wrappers `self` and `second` into [`And`] filter.
    fn and<S: FilterHandler>(self, second: S) -> And
    where
        Self: Sized,
    {
        And {
            first: Arc::new(self),
            second: Arc::new(second),
        }
    }

    /// Wrappers `self` and `other` into [`Or`] filter.
    fn or<O: FilterHandler>(self, other: O) -> Or
    where
        Self: Sized,
    {
        Or {
            first: Arc::new(self),
            other: Arc::new(other),
        }
    }

    /// Wrappers `self` into [`Not`] filter.
    fn not(self) -> Not
    where
        Self: Sized,
    {
        Not {
            filter: Arc::new(self),
        }
    }
}

#[async_trait]
impl<T: ?Sized, F, O: Into<Flow>> FilterHandler for T
where
    T: Fn(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
{
    async fn check(&self, client: Client, update: Update) -> Flow {
        match self(client, update).await.try_into() {
            Ok(flow) => flow,
            Err(_) => flow::break_now(),
        }
    }
}

#[async_trait]
impl<T: ?Sized, F, O: Into<Flow>> FilterHandler for Arc<T>
where
    T: Fn(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
{
    async fn check(&self, client: Client, update: Update) -> Flow {
        match self(client, update).await.try_into() {
            Ok(flow) => flow,
            Err(_) => flow::break_now(),
        }
    }
}
