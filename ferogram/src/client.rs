// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Client traits and helpers.

use std::sync::Arc;

use grammers::{Client, SenderPool, SignInError, client::ClientConfiguration};
use grammers_session::{Session, storages::SqliteSession};

use crate::{error::ClientError, utils::prompt};

/// Extension trait that implements connection helpers to [`Client`].
pub trait ConnectionExt {
    /// Create and connect a new instance from environment variables.
    ///
    /// Uses [`SqliteSession`] as the default session.
    ///
    /// It tries to read following variables:
    /// * `BOT_TOKEN`: bot's token from @BotFather, or
    /// * `PHONE_NUMBER`: user's phone number (international way)
    /// * `API_ID`: developer's API ID from my.telegram.org
    /// * `API_HASH`: developer's API HASH from my.telegram.org
    /// * `SESSION_FILE`: connection's session file.
    fn from_env() -> impl Future<Output = Result<(SenderPool, Client), ClientError>> + Send;

    /// Like [`Self::from_env`] but with a custom [`ClientConfiguration`].
    fn from_env_with_configuration(
        configuration: ClientConfiguration,
    ) -> impl Future<Output = Result<(SenderPool, Client), ClientError>> + Send;

    /// Create and connect a new instance to Telegram servers.
    ///
    /// It doesn't listen to updates, nor dispatches them, you need to
    /// use a [`crate::Dispatcher`] to that.
    ///
    /// Arguments:
    /// * `account`: bot's token from @BotFather or user's phone number (international way)
    /// * `api_id`: developer's API ID from my.telegram.org
    /// * `api_hash`: developer's API HASH from my.telegram.org
    /// * `session`: any object that implements [`Session`]
    fn connect<S: Session + 'static>(
        account: &str,
        api_id: i32,
        api_hash: &str,
        session: Arc<S>,
    ) -> impl Future<Output = Result<(SenderPool, Client), ClientError>> + Send;

    /// Like [`Self::connect`] but with a custom [`ClientConfiguration`].
    fn connect_with_configuration<S: Session + 'static>(
        account: &str,
        api_id: i32,
        api_hash: &str,
        session: Arc<S>,
        configuration: ClientConfiguration,
    ) -> impl Future<Output = Result<(SenderPool, Client), ClientError>> + Send;
}

impl ConnectionExt for Client {
    async fn from_env() -> Result<(SenderPool, Self), ClientError> {
        Self::from_env_with_configuration(Default::default()).await
    }

    async fn from_env_with_configuration(
        configuration: ClientConfiguration,
    ) -> Result<(SenderPool, Self), ClientError> {
        let bot_token = std::env::var("BOT_TOKEN");
        let phone_number = std::env::var("PHONE_NUMBER");

        if bot_token.is_err() && phone_number.is_err() {
            return Err(ClientError::ExpectedVariable(
                "BOT_TOKEN or PHONE_NUMBER".to_string(),
            ));
        }

        let api_id = std::env::var("API_ID")
            .map_err(|_| ClientError::ExpectedVariable("API_ID".to_string()))
            .unwrap()
            .parse::<i32>()
            .unwrap();
        let api_hash = std::env::var("API_HASH")
            .map_err(|_| ClientError::ExpectedVariable("API_HASH".to_string()))
            .unwrap();

        let session_path =
            std::env::var("SESSION_FILE").unwrap_or_else(|_| "grammers.session".to_string());
        let session = Arc::new(SqliteSession::open(session_path).await?);

        let account = bot_token.unwrap_or_else(|_| phone_number.unwrap());
        Self::connect_with_configuration(&account, api_id, &api_hash, session, configuration).await
    }

    async fn connect<S: Session + 'static>(
        account: &str,
        api_id: i32,
        api_hash: &str,
        session: Arc<S>,
    ) -> Result<(SenderPool, Self), ClientError> {
        Self::connect_with_configuration(account, api_id, api_hash, session, Default::default())
            .await
    }

    async fn connect_with_configuration<S: Session + 'static>(
        account: &str,
        api_id: i32,
        api_hash: &str,
        session: Arc<S>,
        configuration: ClientConfiguration,
    ) -> Result<(SenderPool, Self), ClientError> {
        // Test session validity.
        {
            let SenderPool { runner, handle, .. } = SenderPool::new(Arc::clone(&session), api_id);
            let client = Self::new(handle.clone());
            let pool_task = tokio::task::spawn(runner.run());

            if !client.is_authorized().await? {
                if account.contains(":") {
                    client.bot_sign_in(account, api_hash).await?;
                } else {
                    println!("You need to authorize your account. Requesting code...");

                    let token = client.request_login_code(account, api_hash).await?;
                    let code = prompt("Enter the code you received: ", false)?;

                    if let Err(SignInError::PasswordRequired(token)) =
                        client.sign_in(&token, &code).await
                    {
                        let hint = token.hint().unwrap_or_default();
                        let password =
                            prompt(&format!("Enter the password (hint: {hint}): "), true)?;

                        client.check_password(token, password.trim()).await?;
                    }
                }
            }

            handle.quit();
            let _ = pool_task.await;
        }

        // Create a new pool and client that will be returned to the user.
        //
        // It needs to be done like this because `SenderPool` doesn't support
        // reconnecting after been quit.
        let pool = SenderPool::new(Arc::clone(&session), api_id);
        let client = Self::with_configuration(pool.handle.clone(), configuration);

        Ok((pool, client))
    }
}
