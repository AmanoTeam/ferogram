// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

pub mod client;
pub mod utils;

pub use grammers_client;

#[cfg(feature = "macros")]
pub use ferogram_macros as macros;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
