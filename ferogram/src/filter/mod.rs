// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Filters traits and functions intended to be used in [`crate::Handler`]s

mod and;
mod default;
mod markers;
mod not;
mod or;

use std::sync::Arc;

use futures_core::future::BoxFuture;
use grammers::{Client, update::Update};

use crate::{Injector, Result};
pub(crate) use and::AndFilter;
pub use default::*;
pub use markers::*;
pub(crate) use not::NotFilter;
pub(crate) use or::OrFilter;

/// Checks and verifications that runs before any [`crate::Handler`]'s [`crate::di::Endpoint`].
pub trait Filter: Send + Sync + 'static {
    /// Checks if the update should be handled.
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow>;
}

/// Extension trait that implements `.or()`, `.and()` and `.not()` to anything that can be
/// converted into a [`Filter`].
pub trait FilterExt<Marker>: IntoFilter<Marker> + Sized {
    /// Convert this filter into an [`OrFilter`].
    fn or<M2>(self, other: impl IntoFilter<M2>) -> OrFilter
    where
        Self: Sized,
    {
        OrFilter {
            first: Box::new(self.into_filter()),
            other: Box::new(other.into_filter()),
        }
    }

    /// Convert this filter into an [`AndFilter`].
    fn and<M2>(self, other: impl IntoFilter<M2>) -> AndFilter
    where
        Self: Sized,
    {
        AndFilter {
            first: Box::new(self.into_filter()),
            second: Box::new(other.into_filter()),
        }
    }

    /// Convert this filter into a [`NotFilter`].
    fn not(self) -> NotFilter
    where
        Self: Sized,
    {
        NotFilter {
            filter: Box::new(self.into_filter()),
        }
    }
}

impl<Marker, T> FilterExt<Marker> for T where T: IntoFilter<Marker> {}

impl<T: Clone, F, O> Filter for T
where
    T: FnMut(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send,
    O: Into<Flow>,
{
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move { self(client, update).await.into() })
    }
}

impl<T: ?Sized, F, O> Filter for Arc<T>
where
    T: Fn(Client, Update) -> F + Send + Sync + 'static,
    F: Future<Output = O> + Send,
    O: Into<Flow>,
{
    fn run(&mut self, client: &Client, update: &Update) -> BoxFuture<'_, Flow> {
        let client = client.clone();
        let update = update.clone();

        Box::pin(async move { self(client, update).await.into() })
    }
}

/// Control flow of a [`Filter`].
#[derive(Debug, Default)]
pub struct Flow {
    action: FlowAction,
    pub(crate) injector: Injector,
}

impl Flow {
    /// Check if the current action is [`FlowAction::Stop`].
    pub fn is_stop(&self) -> bool {
        self.action == FlowAction::Stop
    }

    /// Check if the current action is [`FlowAction::Proceed`].
    pub fn is_proceed(&self) -> bool {
        self.action == FlowAction::Proceed
    }

    /// Change the current action to [`FlowAction::Stop`].
    pub fn to_stop(&mut self) {
        self.action = FlowAction::Stop
    }

    /// Change the current action to [`FlowAction::Proceed`].
    pub fn to_proceed(&mut self) {
        self.action = FlowAction::Proceed
    }

    /// Inject a new resource.
    pub fn inject<R: Clone + Send + Sync + 'static>(&mut self, value: R) {
        self.injector.push(value);
    }
}

impl From<()> for Flow {
    fn from(_: ()) -> Self {
        stop()
    }
}

impl From<bool> for Flow {
    fn from(value: bool) -> Self {
        match value {
            true => proceed(),
            false => stop(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<Option<T>> for Flow {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => proceed_with(value),
            None => stop(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<Result<T>> for Flow {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(value) => proceed_with(value),
            Err(_) => stop(),
        }
    }
}

/// Next action the filter should execute.
#[derive(Debug, Default, PartialEq)]
pub enum FlowAction {
    Stop,
    #[default]
    Proceed,
}

/// Tell the filter to abort the [`crate::Endpoint`] execution.
pub fn stop() -> Flow {
    Flow {
        action: FlowAction::Stop,
        ..Default::default()
    }
}

/// Tell the filter to execute the [`crate::Endpoint`].
pub fn proceed() -> Flow {
    Flow {
        action: FlowAction::Proceed,
        ..Default::default()
    }
}

/// Tell the filter to execute the [`crate::Endpoint`] with a resource injected.
pub fn proceed_with<R: Clone + Send + Sync + 'static>(value: R) -> Flow {
    let mut injector = Injector::default();
    injector.push(value);

    Flow {
        action: FlowAction::Proceed,
        injector,
    }
}

/// Tell the filter to execute the [`crate::Endpoint`] with all resources injected.
#[cfg(feature = "macros")]
#[macro_export]
macro_rules! proceed_with {
    [$($value:expr),*] => {
        {
            let mut flow = $crate::filter::proceed();

            $(
                flow.inject($value);
            )*

            flow
        }
    };
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_fn_filter() {
        let _: Arc<dyn Filter> = Arc::new(|_: Client, _: Update| async move { Ok(()) });
        let _: Box<dyn Filter> = Box::new(|_: Client, _: Update| async move { Ok(()) });
    }
}
