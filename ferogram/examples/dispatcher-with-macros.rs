//! Example to echo user text message through a dispatcher with client connected from environment variables.
//! Updates are handled concurrently.
//!
//! Based on:
//! [grammers's echo example](https://codeberg.org/Lonami/grammers/src/branch/master/grammers-client/examples/echo.rs).
//!
//! Run it as:
//! ```sh
//! API_ID=... API_HASH="..." BOT_TOKEN="..." PHONE_NUMBER="..." cargo run --example dispatcher
//! ```

use std::error::Error;

use ferogram::prelude::*;
use grammers::{Client, client::UpdatesConfiguration, message::InputMessage, update::Message};

/// You can try it by sending:
/// * `/start 123 hi`: returns id
/// * `/start hi 123`: returns invalid id message
#[handler::new_message(command("/start :id") && text("hi"))]
async fn start(message: Message, params: CommandParams) -> handler::Result {
    let Ok(id) = params.get_parsed::<i64>("id") else {
        let id = params.get("id").unwrap();

        message
            .reply(InputMessage::new().text(format!("Invalid id: {id}")))
            .await?;
        return Ok(());
    };

    message
        .reply(InputMessage::new().text(id.to_string()))
        .await?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting...");

    // Connect the client from environment variables.
    let (pool, client) = Client::from_env().await?;

    // Build and run the dispatcher.
    Dispatcher::builder().add_handler(start()).build().run(
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
