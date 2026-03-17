// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! The main module of the library.

pub mod client;
mod context;
mod di;
mod dispatcher;
pub mod error;
pub mod filter;
pub mod handler;
mod utils;

use std::{error::Error, time::Duration};

use tokio::time::sleep;

pub use context::Context;
pub use di::{Injector, Resource};
pub use dispatcher::Dispatcher;
use dispatcher::STOP_DISPATCHER;
pub use handler::Handler;

pub mod prelude {
    pub use grammers::{self, tl};
    pub use grammers_session as session;

    pub use super::{
        client::*,
        filter::{AsyncMarker, Filter, IntoFilter, SyncMarker},
        *,
    };
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Keep the process alive until a `Ctrl-C` signal is received.
pub async fn wait_for_ctrl_c() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");

    STOP_DISPATCHER.notify_waiters();

    // Sleep to let the dispatcher catch all its tasks.
    sleep(Duration::from_secs(2)).await;
}

/// Same as [`wait_for_ctrl_c`].
pub async fn idle() {
    wait_for_ctrl_c().await
}
