// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

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
pub mod utils;

pub use client::{Client, ClientBuilder as Builder};
pub use dispatcher::Dispatcher;
pub use error_handler::Error;
pub(crate) use error_handler::ErrorHandler;
pub use filter::Filter;
pub(crate) use flow::Flow;
pub(crate) use handler::Handler;
pub use router::Router;

pub use grammers_client as grammers;

#[cfg(feature = "macros")]
#[allow(unused_imports)]
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

/// std [`Result`] with [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
