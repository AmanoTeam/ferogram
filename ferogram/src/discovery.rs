// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Handler discovery through the `inventory` crate.

pub use inventory::submit;

use crate::Handler;

/// A factory that builds a [`Handler`] on demand.
///
/// It is registered at compile time via the `inventory` crate so the
/// [`crate::Dispatcher`] can discover all handlers without manual wiring.
pub struct HandlerFactory(fn() -> Handler);

impl HandlerFactory {
    /// Create a new factory wrapping the given handler constructor.
    pub const fn new(factory: fn() -> Handler) -> Self {
        Self(factory)
    }

    /// Build a fresh [`Handler`] instance.
    pub fn build(&self) -> Handler {
        (self.0)()
    }
}

inventory::collect!(HandlerFactory);
