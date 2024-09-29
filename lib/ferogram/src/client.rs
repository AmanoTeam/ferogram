// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::net::SocketAddr;

use grammers_client::{session::Session, Config, InitParams, SignInError};

use crate::{utils::prompt, Result};

#[derive(Clone)]
pub struct Client {
    pub inner_client: grammers_client::Client,
}

impl Client {
    /// Create a new bot `Client` instance
    pub fn bot(token: impl Into<String>) -> ClientBuilder {
        ClientBuilder::bot(token)
    }

    /// Create a new user `Client` instance
    pub fn user(phone_number: impl Into<String>) -> ClientBuilder {
        ClientBuilder::user(phone_number)
    }
}

pub struct ClientBuilder {
    pub client_type: ClientType,

    pub api_id: i32,
    pub api_hash: String,
    pub session_file: Option<String>,
    pub init_params: InitParams,
}

impl ClientBuilder {
    /// Create a new builder to bot `Client` instance.
    pub fn bot(token: impl Into<String>) -> Self {
        Self {
            client_type: ClientType::Bot(token.into()),

            api_id: 0,
            api_hash: String::new(),
            session_file: None,
            init_params: InitParams::default(),
        }
    }

    /// Create a new builder to user `Client` instance.
    pub fn user(phone_number: impl Into<String>) -> Self {
        Self {
            client_type: ClientType::User(phone_number.into()),

            api_id: 0,
            api_hash: String::new(),
            session_file: None,
            init_params: InitParams::default(),
        }
    }

    /// Build and connect the `Client` instance.
    pub async fn build_and_connect(self) -> Result<Client> {
        let session_file = self
            .session_file
            .unwrap_or("./ferogram.session".to_string());

        let client = grammers_client::Client::connect(Config {
            session: Session::load_file_or_create(&session_file)?,
            api_id: self.api_id,
            api_hash: self.api_hash,
            params: self.init_params,
        })
        .await?;

        if !client.is_authorized().await? {
            match self.client_type {
                ClientType::Bot(token) => match client.bot_sign_in(&token).await {
                    Ok(_) => {
                        client.session().save_to_file(&session_file)?;
                    }
                    Err(e) => {
                        panic!("Failed to sign in: {:?}", e);
                    }
                },
                ClientType::User(phone_number) => {
                    println!("You need to authorize your account. Requesting code...");
                    let token = client.request_login_code(&phone_number).await?;
                    let code = prompt("Enter the code you received: ", false)?;

                    match client.sign_in(&token, &code).await {
                        Ok(_) => {
                            client.session().save_to_file(&session_file)?;
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

        Ok(Client {
            inner_client: client,
        })
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
    pub fn api_hash(mut self, api_hash: impl Into<String>) -> Self {
        self.api_hash = api_hash.into();
        self
    }

    /// Session storage where data should persist, such as authorization key, server address,
    /// and other required information by the client.
    pub fn session_file(mut self, path: impl Into<String>) -> Self {
        self.session_file = Some(path.into());
        self
    }

    /// User's device model.
    ///
    /// Telegram uses to know your device in devices settings.
    pub fn device_model(mut self, device_model: impl Into<String>) -> Self {
        self.init_params.device_model = device_model.into();
        self
    }

    /// User's system version.
    ///
    /// Telegram uses to know your system version in devices settings.
    pub fn system_version(mut self, system_version: impl Into<String>) -> Self {
        self.init_params.system_version = system_version.into();
        self
    }

    /// Client's app version.
    ///
    /// Telegram uses to know your app version in device settings.
    pub fn app_version(mut self, app_version: impl Into<String>) -> Self {
        self.init_params.app_version = app_version.into();
        self
    }

    /// Client's language code.
    ///
    /// Telegram uses internally to let others know your language.
    pub fn lang_code(mut self, lang_code: impl Into<String>) -> Self {
        self.init_params.lang_code = lang_code.into();
        self
    }

    /// Should the client catch-up on updates sent to it while it was offline?
    ///
    /// By default, updates sent while the client was offline are ignored.
    pub fn catch_up(mut self, catch_up: bool) -> Self {
        self.init_params.catch_up = catch_up;
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
}

pub enum ClientType {
    Bot(String),
    User(String),
}
