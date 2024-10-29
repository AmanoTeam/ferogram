// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::{net::SocketAddr, path::Path, sync::Arc};

use grammers_client::{session::Session, Config, InitParams, SignInError};

use crate::{utils::prompt, Dispatcher, Result};

/// Wrapper about grammers' `Client` instance.
pub struct Client {
    client_type: ClientType,
    inner_client: grammers_client::Client,

    dispatcher: Dispatcher,
    is_connected: bool,
    session_file: Option<String>,
    wait_for_ctrl_c: bool,
}

impl Client {
    /// Create a new bot `Client` instance.
    pub fn bot<T: Into<String>>(token: T) -> ClientBuilder {
        ClientBuilder::bot(token)
    }

    /// Create a new user `Client` instance.
    pub fn user<N: Into<String>>(phone_number: N) -> ClientBuilder {
        ClientBuilder::user(phone_number)
    }

    /// Create a new `Client` instance from environment variables.
    ///
    /// It try to read the following env variables:
    ///
    /// * `BOT_TOKEN`: bot's token from @BotFather, or
    /// * `PHONE_NUMBER`: user's phone number (international way)
    /// * `API_ID`: developer's API ID from my.telegram.org
    /// * `API_HASH`: developer's API HASH from my.telegram.org
    pub fn from_env() -> ClientBuilder {
        let mut builder = if let Ok(token) = std::env::var("BOT_TOKEN") {
            Self::bot(token)
        } else if let Ok(phone_number) = std::env::var("PHONE_NUMBER") {
            Self::user(phone_number)
        } else {
            panic!("You need to set BOT_TOKEN or PHONE_NUMBER env variable.");
        };

        match std::env::var("API_ID") {
            Ok(api_id) => builder = builder.api_id(api_id.parse::<i32>().expect("API_ID invalid.")),
            Err(_) => panic!("You need to set API_ID env variable."),
        }

        match std::env::var("API_HASH") {
            Ok(api_hash) => builder = builder.api_hash(api_hash),
            Err(_) => panic!("You need to set API_HASH env variable."),
        }

        builder
    }

    /// Connects to the Telegram server, but don't listen to updates.
    pub async fn connect(mut self) -> Result<Self> {
        if self.is_connected {
            return Err("Client is already connected.".into());
        }

        let session_file = &self.session_file.as_deref().unwrap_or("./ferogram.session");

        let client = &self.inner_client;
        if !client.is_authorized().await? {
            match self.client_type {
                ClientType::Bot(ref token) => match client.bot_sign_in(&token).await {
                    Ok(_) => {
                        client.session().save_to_file(session_file)?;
                    }
                    Err(e) => {
                        panic!("Failed to sign in: {:?}", e);
                    }
                },
                ClientType::User(ref phone_number) => {
                    println!("You need to authorize your account. Requesting code...");
                    let token = client.request_login_code(&phone_number).await?;
                    let code = prompt("Enter the code you received: ", false)?;

                    match client.sign_in(&token, &code).await {
                        Ok(_) => {
                            client.session().save_to_file(session_file)?;
                        }
                        Err(SignInError::PasswordRequired(token)) => {
                            let hint = token.hint().unwrap();
                            let password =
                                prompt(format!("Enter the password (hint {}): ", hint), true)?;

                            client.check_password(token, password.trim()).await?;
                        }
                        Err(e) => {
                            panic!("Failed to sign in: {:?}", e);
                        }
                    }
                }
            };
        }
        self.is_connected = true;

        Ok(self)
    }

    /// Configure the dispatcher.
    pub fn dispatcher<D: FnOnce(Dispatcher) -> Dispatcher>(mut self, dispatcher: D) -> Self {
        self.dispatcher = dispatcher(self.dispatcher);
        self
    }

    /// Wheter the client is connected.
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Listen to Telegram's updates and send them to the dispatcher's routers.
    pub async fn run(self) -> Result<()> {
        let handle = self.inner_client;
        let dispatcher = Arc::new(self.dispatcher);

        tokio::task::spawn(async move {
            loop {
                match handle.next_update().await {
                    Ok(update) => {
                        let client = handle.clone();
                        let dispatcher = Arc::clone(&dispatcher);

                        tokio::task::spawn(async move {
                            match dispatcher.handle_update(client, update).await {
                                Ok(_) => {}
                                Err(e) => {
                                    log::error!("Error handling update: {:?}", e);
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("Error getting updates: {:?}", e);
                    }
                }
            }
        });

        if self.wait_for_ctrl_c {
            tokio::signal::ctrl_c().await?;
        }

        Ok(())
    }

    /// Keeps the connection open, but doesn't listen to the updates.
    pub async fn keep_alive(self) -> Result<()> {
        let client = self.inner_client;

        tokio::task::spawn(async move {
            loop {
                client.step().await.unwrap();
            }
        });

        if self.wait_for_ctrl_c {
            tokio::signal::ctrl_c().await?;
        }

        Ok(())
    }
}

/// `Client` instance builder.
#[derive(Default)]
pub struct ClientBuilder {
    client_type: ClientType,

    api_id: i32,
    api_hash: String,
    session_file: Option<String>,
    init_params: InitParams,

    wait_for_ctrl_c: bool,
}

impl ClientBuilder {
    /// Create a new builder to bot `Client` instance.
    pub fn bot<T: Into<String>>(token: T) -> Self {
        Self {
            client_type: ClientType::Bot(token.into()),

            ..Default::default()
        }
    }

    /// Create a new builder to user `Client` instance.
    pub fn user<N: Into<String>>(phone_number: N) -> Self {
        Self {
            client_type: ClientType::User(phone_number.into()),

            ..Default::default()
        }
    }

    /// Build the `Client` instance.
    pub async fn build(self) -> Result<Client> {
        let session_file = self.session_file.as_deref().unwrap_or("./ferogram.session");

        let inner_client = grammers_client::Client::connect(Config {
            session: Session::load_file_or_create(session_file)?,
            api_id: self.api_id,
            api_hash: self.api_hash,
            params: self.init_params,
        })
        .await?;

        Ok(Client {
            client_type: self.client_type,
            inner_client,

            dispatcher: Dispatcher::default(),
            is_connected: false,
            session_file: Some(session_file.to_string()),
            wait_for_ctrl_c: self.wait_for_ctrl_c,
        })
    }

    /// Build and connect the `Client` instance.
    ///
    /// Connects to the Telegram server, but don't listen to updates.
    pub async fn build_and_connect(self) -> Result<Client> {
        let session_file = self.session_file.as_deref().unwrap_or("./ferogram.session");

        let client = grammers_client::Client::connect(Config {
            session: Session::load_file_or_create(session_file)?,
            api_id: self.api_id,
            api_hash: self.api_hash,
            params: self.init_params,
        })
        .await?;

        Ok(Client {
            client_type: self.client_type,
            inner_client: client,

            dispatcher: Dispatcher::default(),
            is_connected: false,
            session_file: Some(session_file.to_string()),
            wait_for_ctrl_c: self.wait_for_ctrl_c,
        }
        .connect()
        .await?)
    }

    /// Developer's API ID, required to interact with the Telegram's API.
    ///
    /// You may obtain your own in <https://my.telegram.org/auth>.
    pub fn api_id(mut self, api_id: i32) -> Self {
        self.api_id = api_id;
        self
    }

    /// Developer's API hash, required to interact with Telegram's API.
    ///
    /// You may obtain your own in <https://my.telegram.org/auth>.
    pub fn api_hash<H: Into<String>>(mut self, api_hash: H) -> Self {
        self.api_hash = api_hash.into();
        self
    }

    /// Session storage where data should persist, such as authorization key, server address,
    /// and other required information by the client.
    pub fn session_file<P: AsRef<Path> + ToString>(mut self, path: P) -> Self {
        self.session_file = Some(path.to_string());
        self
    }

    /// User's device model.
    ///
    /// Telegram uses to know your device in devices settings.
    pub fn device_model<M: Into<String>>(mut self, device_model: M) -> Self {
        self.init_params.device_model = device_model.into();
        self
    }

    /// User's system version.
    ///
    /// Telegram uses to know your system version in devices settings.
    pub fn system_version<V: Into<String>>(mut self, system_version: V) -> Self {
        self.init_params.system_version = system_version.into();
        self
    }

    /// Client's app version.
    ///
    /// Telegram uses to know your app version in device settings.
    pub fn app_version<V: Into<String>>(mut self, app_version: V) -> Self {
        self.init_params.app_version = app_version.into();
        self
    }

    /// Client's language code.
    ///
    /// Telegram uses internally to let others know your language.
    pub fn lang_code<C: Into<String>>(mut self, lang_code: C) -> Self {
        self.init_params.lang_code = lang_code.into();
        self
    }

    /// Should the client catch-up on updates sent to it while it was offline?
    ///
    /// By default, updates sent while the client was offline are ignored.
    pub fn catch_up(mut self, value: bool) -> Self {
        self.init_params.catch_up = value;
        self
    }

    /// Server address to connect to. By default, the library will connect to the address stored
    /// in the session file (or a default production address if no such address exists). This
    /// field can be used to override said address, and is most commonly used to connect to one
    /// of Telegram's test servers instead.
    pub fn server_address(mut self, server_address: SocketAddr) -> Self {
        self.init_params.server_addr = Some(server_address);
        self
    }

    /// The threshold below which the library should automatically sleep on flood-wait and slow
    /// mode wait errors (inclusive). For instance, if an
    /// `RpcError { name: "FLOOD_WAIT", value: Some(17) }` (flood, must wait 17 seconds) occurs
    /// and `flood_sleep_threshold` is 20 (seconds), the library will `sleep` automatically for
    /// 17 seconds. If the error was for 21s, it would propagate the error instead.
    ///
    /// By default, the library will sleep on flood-waits below or equal to one minute (60
    /// seconds), but this can be disabled by passing `0` (since all flood errors would be
    /// higher and exceed the threshold).
    ///
    /// On flood, the library will retry *once*. If the flood error occurs a second time after
    /// sleeping, the error will be returned.
    pub fn flood_sleep_threshold(mut self, flood_sleep_threshold: u32) -> Self {
        self.init_params.flood_sleep_threshold = flood_sleep_threshold;
        self
    }

    /// How many updates may be buffered by the client at any given time.
    ///
    /// Telegram passively sends updates to the client through the open connection, so they must
    /// be buffered until the application has the capacity to consume them.
    ///
    /// Upon reaching this limit, updates will be dropped, and a warning log message will be
    /// emitted (but not too often, to avoid spamming the log), in order to let the developer
    /// know that they should either change how they handle updates or increase the limit.
    ///
    /// A limit of zero (`0`) indicates that updates should not be buffered. They will be
    /// immediately dropped, and no warning will ever be emitted.
    ///
    /// A limit of `None` disables the upper bound for the buffer. This is not recommended, as it
    /// could eventually lead to memory exhaustion. This option will also not emit any warnings.
    ///
    /// The default limit, which may change at any time, should be enough for user accounts,
    /// although bot accounts may need to increase the limit depending on their capacity.
    ///
    /// When the limit is `Some`, a buffer to hold that many updates will be pre-allocated.
    pub fn update_queue_limit(mut self, update_queue_limit: Option<usize>) -> Self {
        self.init_params.update_queue_limit = update_queue_limit;
        self
    }

    /// Wait for `Ctrl + C` to exit the app.
    ///
    /// Otherwise the code will continue running until it finds the end.
    pub fn wait_for_ctrl_c(mut self) -> Self {
        self.wait_for_ctrl_c = true;
        self
    }
}

/// Client type.
#[derive(Clone)]
pub enum ClientType {
    /// Bot client, holds bot token.
    Bot(String),
    /// User client, holds user phone number.
    User(String),
}

impl Default for ClientType {
    fn default() -> Self {
        Self::Bot(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_bot() {
        let client = Client::bot(std::env::var("BOT_TOKEN").unwrap())
            .api_id(std::env::var("API_ID").unwrap().parse::<i32>().unwrap())
            .api_hash(std::env::var("API_HASH").unwrap())
            .build_and_connect()
            .await
            .unwrap();

        assert_eq!(client.inner_client.is_authorized().await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_client_user() {
        let client = Client::user(std::env::var("PHONE_NUMBER").unwrap())
            .api_id(std::env::var("API_ID").unwrap().parse::<i32>().unwrap())
            .api_hash(std::env::var("API_HASH").unwrap())
            .build_and_connect()
            .await
            .unwrap();

        assert_eq!(client.inner_client.is_authorized().await.unwrap(), true);
    }
}
