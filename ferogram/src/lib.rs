// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! The main module of the library.

pub mod client;
mod context;
pub mod di;
mod dispatcher;
pub mod error;
pub mod filter;
pub mod handler;
pub mod router;
mod utils;

use std::error::Error;

pub use context::Context;
use di::Injector;
pub use dispatcher::Dispatcher;
use dispatcher::{DISPATCHER_STOPPED, STOP_DISPATCHER};
pub use handler::Handler;

/// It reexports useful and necessary traits and utilities.
///
/// It also reexports some [`grammes`] libraries as:
/// * `tl`: [`grammers-tl-types`]
/// * `grammers`: [`grammers-client`].
/// * `session`: [`grammers-session`]
pub mod prelude {
    pub use grammers::{self, tl};
    pub use grammers_session as session;

    pub use super::{
        client::ConnectionExt,
        di::Resource,
        filter::{self, AsyncMarker, Filter, FilterExt, IntoFilter, SyncMarker, default::*},
        router::CommandParams,
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

/// Keep the process alive until a `Ctrl-C` signal is received, or until the
/// dispatcher stops.
///
/// Upon receiving a `Ctrl-C` signal, it signals the dispatcher to begina graceful
/// shutdown. Unlike [`self::wait_for_ctrl_c`], it waits for the dispatcher to
/// successfully save the session and for all spawned handler tasks to finish.
///
/// Note: If the background dispatcher encounters a fatal panic or exits unexpectedly,
/// it will return immediately to prevent the application from hanging.
pub async fn idle() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Ctrl-C received. Instructing dispatcher to stop...");
            STOP_DISPATCHER.notify_one();

            DISPATCHER_STOPPED.notified().await;
            tracing::info!("Application stopped successfully.");
        }
        _ = DISPATCHER_STOPPED.notified() => {
            tracing::error!("Idle aborted: The dispatcher stopped unexpectedly.");
        }
    }
}
