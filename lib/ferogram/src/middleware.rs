// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Middleware module.

use std::future::Future;

use async_trait::async_trait;
use grammers_client::{Client, Update};

use crate::{Flow, Injector};

/// A stack of middlewares.
#[derive(Clone, Default)]
pub struct MiddlewareStack {
    pub(crate) after: Vec<Box<dyn Middleware>>,
    pub(crate) before: Vec<Box<dyn Middleware>>,
}

impl MiddlewareStack {
    /// Creates a new middleware stack.
    pub fn new() -> MiddlewareStack {
        MiddlewareStack {
            after: Vec::new(),
            before: Vec::new(),
        }
    }

    /// Adds a middleware after-type in the stack.
    pub fn after<M: Middleware>(mut self, middleware: M) -> Self {
        self.after.push(Box::new(middleware));
        self
    }

    /// Adds a middleware before-type in the stack.
    pub fn before<M: Middleware>(mut self, middleware: M) -> Self {
        self.before.push(Box::new(middleware));
        self
    }

    /// Extends the middleware stack with another middleware stack.
    pub(crate) fn extend(mut self, other: MiddlewareStack) -> Self {
        self.after.extend(other.after);
        self.before.extend(other.before);
        self
    }

    /// Handles the after-type middlewares.
    pub(crate) async fn handle_after(
        &mut self,
        client: &Client,
        update: &Update,
        injector: &mut Injector,
    ) {
        for middleware in self.after.iter_mut() {
            let flow = middleware.handle(client, update, injector).await;
            if flow.is_break() {
                break;
            }
        }
    }

    /// Handles the before-type middlewares.
    pub(crate) async fn handle_before(
        &mut self,
        client: &Client,
        update: &Update,
        injector: &mut Injector,
    ) -> Flow {
        let mut flow = Flow::default();

        for middleware in self.before.iter_mut() {
            flow = middleware.handle(client, update, injector).await;
            if flow.is_break() {
                break;
            }
        }

        flow
    }
}

#[async_trait]
/// Middleware trait.
pub trait Middleware: CloneMiddleware + Send + Sync + 'static {
    /// Handles the middleware.
    async fn handle(&mut self, client: &Client, update: &Update, injector: &mut Injector) -> Flow;
}

#[async_trait]
impl<Fut: Clone, Output> Middleware for Fut
where
    Fut: for<'a> FnMut(&'a Client, &'a Update, &'a mut Injector) -> Output + Send + Sync + 'static,
    Output: Future<Output = Flow> + Send,
{
    async fn handle(&mut self, client: &Client, update: &Update, injector: &mut Injector) -> Flow {
        self(client, update, injector).await.into()
    }
}

/// A trait that allows cloning the middleware.
pub trait CloneMiddleware {
    /// Clones the middleware.
    fn clone_middleware(&self) -> Box<dyn Middleware>;
}

impl<T> CloneMiddleware for T
where
    T: Middleware + Clone + 'static,
{
    fn clone_middleware(&self) -> Box<dyn Middleware> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Middleware> {
    fn clone(&self) -> Self {
        self.clone_middleware()
    }
}
