// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Ferogram is a small framework for building Telegram bots using the [`grammers`] library.
//!
//! The main module of the library.

mod client;
pub(crate) mod di;
mod dispatcher;
mod error_handler;
pub mod filter;
pub(crate) mod filters;
pub mod flow;
pub mod handler;
mod router;
pub(crate) mod utils;

#[cfg(feature = "plugins")]
mod plugins;

pub use client::{Client, ClientBuilder as Builder};
pub use di::Injector;
pub use dispatcher::Dispatcher;
pub use error_handler::Error;
pub(crate) use error_handler::ErrorHandler;
pub use filter::Filter;
pub(crate) use flow::Flow;
pub(crate) use handler::Handler;
pub use router::Router;

#[cfg(feature = "plugins")]
pub use plugins::Plugin;

#[cfg(feature = "macros")]
pub use ferogram_macros as macros;

#[cfg(feature = "macros")]
/// Construct [`di::Injector`] with a list of dependencies effortlessly.
///
/// # Example
///
/// ```
/// deps![Database::connect().await, I18n::load()]
/// ```
#[macro_export]
macro_rules! deps {
        [$($dep:expr),*] => {
            |injector| { injector$(.with($dep))* }
        };
    }

/// Common types and traits.
pub mod prelude {
    pub use crate::{
        filter::{and, not, or},
        *,
    };
}

/// [`Result`] with [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Wait for a `Ctrl+C` signal and keep the process alive.
pub async fn wait_for_ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}

/// Wait for a `Ctrl+C` signal and keep the process alive.
///
/// Same as [`wait_for_ctrl_c`].
pub async fn idle() {
    wait_for_ctrl_c().await
}
