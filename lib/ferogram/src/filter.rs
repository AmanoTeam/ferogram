// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Filters module.

use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use futures_util::Future;
use grammers_client::{Client, Update};

use crate::Flow;
pub use crate::filters::*;

/// A filter.
#[async_trait]
pub trait Filter: CloneFilter + Send + Sync + 'static {
    /// Checks if the update should be handled.
    async fn check(&mut self, client: &Client, update: &Update) -> Flow;

    /// Wrappes `self` and `second` into [`And`] filter.
    fn and<S: Filter>(self, second: S) -> And
    where
        Self: Sized,
    {
        And {
            first: Box::new(self),
            second: Box::new(second),
        }
    }

    /// Wrappes `self` and `other` into [`Or`] filter.
    fn or<O: Filter>(self, other: O) -> Or
    where
        Self: Sized,
    {
        Or {
            first: Box::new(self),
            other: Box::new(other),
        }
    }

    /// Wrappes `self` into [`Not`] filter.
    fn not(self) -> Not
    where
        Self: Sized,
    {
        Not {
            filter: Box::new(self),
        }
    }

    /// Returns the filter as a `Any` trait object.
    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }
}

#[async_trait]
impl<T: Clone, F, O> Filter for T
where
    T: FnMut(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
    O: Into<Flow>,
{
    async fn check(&mut self, client: &Client, update: &Update) -> Flow {
        self(client.clone(), update.clone()).await.into()
    }
}

#[async_trait]
impl<T: ?Sized, F, O> Filter for Arc<T>
where
    T: Fn(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
    O: Into<Flow>,
{
    async fn check(&mut self, client: &Client, update: &Update) -> Flow {
        self(client.clone(), update.clone()).await.into()
    }
}

/// A trait that allows cloning the filter.
pub trait CloneFilter {
    /// Clones the filter.
    fn clone_filter(&self) -> Box<dyn Filter>;
}

impl<T> CloneFilter for T
where
    T: Filter + Clone + 'static,
{
    fn clone_filter(&self) -> Box<dyn Filter> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Filter> {
    fn clone(&self) -> Self {
        self.clone_filter()
    }
}
