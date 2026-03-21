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

use std::error::Error;

pub use context::Context;
pub use di::{Injector, Resource};
pub use dispatcher::Dispatcher;
use dispatcher::{DISPATCHER_STOPPED, STOP_DISPATCHER};
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
        .expect("Failed to listen for Ctrl+C signal")
}

/// Keep the process alive until a `Ctrl-C` signal is received.
///
/// Unlike [`self::wait_for_ctrl_c`], it waits for all handler tasks spawned by
/// the dispatcher to finish.
pub async fn idle() {
    wait_for_ctrl_c().await;

    STOP_DISPATCHER.notify_waiters();
    DISPATCHER_STOPPED.notified().await;
}
