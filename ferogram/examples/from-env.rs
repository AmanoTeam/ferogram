//! Example to echo user text message with client connected from environment variables.
//! Updates are handled concurrently.
//!
//! Based on:
//! [grammers's echo example](https://codeberg.org/Lonami/grammers/src/branch/master/grammers-client/examples/echo.rs).
//!
//! Run it as:
//! ```sh
//! API_ID=... API_HASH="..." BOT_TOKEN="..." PHONE_NUMBER="..." cargo run --example from-env
//! ```

use std::{error::Error, time::Duration};

use ferogram::prelude::ConnectionExt;
use grammers::{Client, SenderPool, client::UpdatesConfiguration, update::Update};
use tokio::{task::JoinSet, time::sleep};

async fn handle_update(client: Client, update: Update) {
    match update {
        Update::NewMessage(message) if !message.outgoing() => {
            let peer = message.peer().unwrap();
            println!(
                "Responding to {}",
                peer.name().unwrap_or(&format!("id {}", message.peer_id()))
            );
            if message.text() == "slow" {
                sleep(Duration::from_secs(5)).await;
            }
            if let Err(e) = client
                .send_message(peer.to_ref().await.unwrap(), message.text())
                .await
            {
                println!("Failed to respond! {e}");
            };
        }
        _ => {}
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting...");

    // Connect the client from environment variables.
    let (pool, client) = Client::from_env().await?;

    let SenderPool {
        runner,
        handle,
        updates,
    } = pool;
    let pool_task = tokio::spawn(runner.run());

    println!("Waiting for messages...");

    // This example spawns a task to handle each update.
    // To guarantee that all handlers run to completion, they're stored in this set.
    // You can use `task::spawn` if you don't care about dropping unfinished handlers midway.
    let mut handler_tasks = JoinSet::new();
    let mut updates = client
        .stream_updates(
            updates,
            UpdatesConfiguration {
                catch_up: true,
                ..Default::default()
            },
        )
        .await;

    loop {
        // Empty finished handlers (you could look at their return value here too.)
        while handler_tasks.try_join_next().is_some() {}

        // This example uses `select` on Ctrl+C to gracefully stop the client and have a chance to
        // save the session. You could have fancier logic to save the session if you wanted to
        // (or even save it on every update). Or you could also ignore Ctrl+C and just use
        // `let update = client.next_update().await?`.
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            update = updates.next() => {
                let update = update?;
                let handle = client.clone();

                handler_tasks.spawn(handle_update(handle, update));
            }
        }
    }

    println!("Saving session file...");
    updates.sync_update_state().await;

    // Pool's `run()` won't finish until all handles are dropped or quit is called.
    // Here there are at least three handles alive: `handle`, `client` and `updates`
    // which contains a `client`. Any ongoing `handle_update` handlers have one client too.
    // In this case, it's easier to call `handle.quit()` to close them all.
    //
    // You don't need to explicitly close the connection, but this is a way to do it gracefully.
    // This also gives a chance to the handlers to finish their work by handling the `Dropped`
    // error from any pending method calls (RPC invocations).
    //
    // You can try this graceful shutdown by sending a message saying "slow" and then pressing Ctrl+C.
    println!("Gracefully closing connection to notify all pending handlers...");
    handle.quit();
    let _ = pool_task.await;

    // Give a chance to all on-going handlers to finish.
    println!("Waiting for any slow handlers to finish...");
    while handler_tasks.try_join_next().is_some() {}

    Ok(())
}
