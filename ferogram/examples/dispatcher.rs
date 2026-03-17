//! Example to echo user text message through a dispatcher with client connected from environment variables.
//! Updates are handled concurrently.
//!
//! Based on:
//! [grammers's echo example](https://codeberg.org/Lonami/grammers/src/branch/master/grammers-client/examples/echo.rs).
//!
//! Run it as:
//! ```sh
//! API_ID=... API_HASH="..." BOT_TOKEN="..." PHONE_NUMBER="..." cargo run --example from-env
//! ```

use std::error::Error;

use ferogram::{Dispatcher, filter, handler, prelude::ConnectionExt};
use grammers::{Client, client::UpdatesConfiguration, update::Update};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting...");

    // Connect the client from environment variables.
    let (pool, client) = Client::from_env().await?;

    // Build and run the dispatcher.
    Dispatcher::builder()
        .add_handler(
            handler::new_message(filter::text("hi")).then(|update: Update| async {
                if let Update::NewMessage(message) = update {
                    message.reply(message.text()).await?;
                }

                Ok(())
            }),
        )
        .build()
        .run(
            pool,
            client,
            UpdatesConfiguration {
                catch_up: true,
                ..Default::default()
            },
        );

    println!("Waiting for messages...");

    // Idle so the app can continue running.
    ferogram::idle().await;

    Ok(())
}
