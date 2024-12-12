// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Update error filter.

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::Future;
use grammers_client::{Client, Update};

use crate::{flow, Flow};

/// [`Error`] boxed.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Update error filter.
///
/// Runned when the handler's endpoint returns an error.
#[async_trait]
pub trait ErrorHandler: CloneErrorHandler + Send + Sync + 'static {
    /// Run the error handler.
    async fn run(&self, client: Client, update: Update, error: Error) -> Flow;
}

#[async_trait]
impl<T: Clone + ?Sized, F, O: Into<Flow>> ErrorHandler for T
where
    T: Fn(Client, Update, Error) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
{
    async fn run(&self, client: Client, update: Update, error: Error) -> Flow {
        match self(client, update, error).await.try_into() {
            Ok(flow) => flow,
            Err(_) => flow::break_now(),
        }
    }
}

#[async_trait]
impl<T: ?Sized, F, O: Into<Flow>> ErrorHandler for Arc<T>
where
    T: Fn(Client, Update, Error) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send + Sync + 'static,
{
    async fn run(&self, client: Client, update: Update, error: Error) -> Flow {
        match self(client, update, error).await.try_into() {
            Ok(flow) => flow,
            Err(_) => flow::break_now(),
        }
    }
}

/// A trait that allows cloning the error handler.
pub trait CloneErrorHandler {
    /// Clones the error handler.
    fn clone_error_handler(&self) -> Box<dyn ErrorHandler>;
}

impl<T> CloneErrorHandler for T
where
    T: ErrorHandler + Clone,
{
    fn clone_error_handler(&self) -> Box<dyn ErrorHandler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ErrorHandler> {
    fn clone(&self) -> Self {
        self.clone_error_handler()
    }
}
