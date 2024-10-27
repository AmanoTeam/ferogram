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
pub mod filter;
pub(crate) mod filters;
pub mod handler;
mod router;
pub mod utils;

pub use client::Client;
pub use dispatcher::Dispatcher;
pub(crate) use filter::Filter;
pub(crate) use handler::Handler;
pub use router::Router;

pub use grammers_client as grammers;

#[cfg(feature = "macros")]
pub use ferogram_macros as macros;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
