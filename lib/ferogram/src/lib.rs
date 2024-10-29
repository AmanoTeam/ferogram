// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

pub mod client;
pub(crate) mod di;
mod dispatcher;
mod error_handler;
pub mod filter;
pub(crate) mod filters;
pub mod flow;
pub mod handler;
mod router;
pub mod utils;

pub use client::Client;
pub use dispatcher::Dispatcher;
pub use error_handler::Error;
pub(crate) use error_handler::ErrorHandler;
pub use filter::Filter;
pub use flow::Action;
pub(crate) use flow::Flow;
pub(crate) use handler::Handler;
pub use router::Router;

pub use grammers_client as grammers;

#[cfg(feature = "macros")]
pub use ferogram_macros as macros;

/// Common types and traits.
pub mod prelude {
    pub use super::{
        filter::{and, not, or},
        *,
    };
}

pub type Result<T> = std::result::Result<T, Error>;
