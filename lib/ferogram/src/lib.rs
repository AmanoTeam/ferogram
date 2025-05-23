// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Ferogram is a small framework for building Telegram bots using the [`grammers`] library.
//!
//! The main module of the library.

mod cache;
mod client;
mod context;
pub(crate) mod di;
mod dispatcher;
pub mod error;
mod error_handler;
pub mod filter;
pub(crate) mod filters;
pub mod flow;
pub mod handler;
mod middleware;
mod plugin;
mod router;
pub mod utils;

pub(crate) use cache::Cache;
pub use client::{Client, ClientBuilder as Builder};
pub use context::Context;
pub use di::Injector;
pub use dispatcher::Dispatcher;
pub use error::Error;
pub(crate) use error_handler::ErrorHandler;
pub use filter::Filter;
pub(crate) use flow::Flow;
pub(crate) use handler::Handler;
pub use middleware::{Middleware, MiddlewareStack};
pub use plugin::Plugin;
pub use router::Router;

#[cfg(feature = "lua")]
pub mod lua;

#[cfg(feature = "python")]
pub mod py;

#[cfg(feature = "macros")]
pub use ferogram_macros as macros;

#[cfg(feature = "macros")]
/// Constructs a [`di::Injector`] with a list of dependencies effortlessly.
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

/// [`Result`] with [`Error`].
pub type Result<T> = std::result::Result<T, error_handler::Error>;

/// Waits for a `Ctrl+C` signal and keep the process alive.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// ferogram::wait_for_ctrl_c().await;
/// # }
/// ```
pub async fn wait_for_ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}

/// Waits for a `Ctrl+C` signal and keep the process alive.
///
/// Same as [`wait_for_ctrl_c`].
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// ferogram::idle().await;
/// # }
/// ```
pub async fn idle() {
    wait_for_ctrl_c().await
}
