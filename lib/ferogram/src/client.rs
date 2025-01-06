// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Client module.

use std::path::Path;

use grammers_client::{
    grammers_tl_types as tl, session::Session, Config, InitParams, ReconnectionPolicy, SignInError,
};
use grammers_mtsender::ServerAddr;

use crate::{di, utils::prompt, Context, Dispatcher, ErrorHandler, Result};

/// Wrapper about grammers' `Client` instance.
pub struct Client {
    /// The dispatcher.
    dispatcher: Dispatcher,
    /// The client type.
    client_type: ClientType,
    /// The inner grammers' `Client` instance.
    inner_client: grammers_client::Client,

    /// The session file path.
    session_file: Option<String>,

    /// Whether the client is connected.
    is_connected: bool,
    /// Whether is to update Telegram's bot commands.
    set_bot_commands: bool,
    /// Wheter is to wait for a `Ctrl + C` signal to close the connection and exit the app.
    wait_for_ctrl_c: bool,

    /// The global error handler.
    pub(crate) err_handler: Option<Box<dyn ErrorHandler>>,
    /// The exit handler.
    pub(crate) exit_handler: Option<di::Endpoint>,
    /// The ready handler.
    pub(crate) ready_handler: Option<di::Endpoint>,
}

impl Client {
    /// Creates a new bot instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ferogram::Client;
    /// #
    /// # async fn example() {
    /// let client = Client::bot(std::env::var("BOT_TOKEN").unwrap_or_default()).build().await?;
    /// # }
    /// ```
    pub fn bot<T: Into<String>>(token: T) -> ClientBuilder {
        ClientBuilder::bot(token)
    }

    /// Creates a new user instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ferogram::Client;
    /// #
    /// # async fn example() {
    /// let client = Client::user(std::env::var("PHONE_NUMBER").unwrap()).build().await?;
    /// # }
    /// ```
    pub fn user<N: Into<String>>(phone_number: N) -> ClientBuilder {
        ClientBuilder::user(phone_number)
    }

    /// Creates a new `Client` instance from environment variables.
    ///
    /// It try to read the following env variables:
    ///
    /// * `BOT_TOKEN`: bot's token from @BotFather, or
    /// * `PHONE_NUMBER`: user's phone number (international way)
    /// * `API_ID`: developer's API ID from my.telegram.org
    /// * `API_HASH`: developer's API HASH from my.telegram.org
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ferogram::Client;
    /// #
    /// # async fn example() {
    /// let client = Client::from_env().build().await?;
    /// # }
    /// ```
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.connect().await?;
    /// # }
    /// ```
    pub async fn connect(mut self) -> Result<Self> {
        if self.is_connected {
            return Err("Client is already connected.".into());
        }

        let session_file = self.session_file.as_deref().unwrap_or("./ferogram.session");

        let client = &self.inner_client;
        if !client.is_authorized().await? {
            match self.client_type {
                ClientType::Bot(ref token) => match client.bot_sign_in(token).await {
                    Ok(_) => {
                        client.session().save_to_file(session_file)?;
                    }
                    Err(e) => {
                        panic!("Failed to sign in: {:?}", e);
                    }
                },
                ClientType::User(ref phone_number) => {
                    println!("You need to authorize your account. Requesting code...");
                    let token = client.request_login_code(phone_number).await?;
                    let code = prompt("Enter the code you received: ", false)?;

                    match client.sign_in(&token, &code).await {
                        Ok(_) => {
                            client.session().save_to_file(session_file)?;
                        }
                        Err(SignInError::PasswordRequired(token)) => {
                            let hint = token.hint().unwrap();
                            let password =
                                prompt(format!("Enter the password (hint: {}): ", hint), true)?;

                            if client.check_password(token, password.trim()).await.is_ok() {
                                client.session().save_to_file(session_file)?;
                            }
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

    /// Gets the inner grammers' `Client` instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.inner();
    /// # }
    /// ```
    pub fn inner(&self) -> &grammers_client::Client {
        &self.inner_client
    }

    /// Configures the dispatcher.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.dispatcher(|dispatcher| {
    ///     dispatcher
    /// });
    /// # }
    /// ```
    pub fn dispatcher<D: FnOnce(Dispatcher) -> Dispatcher>(mut self, dispatcher: D) -> Self {
        self.dispatcher = dispatcher(self.dispatcher);
        self
    }

    /// Whether the client is connected.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let is_connected = client.is_connected();
    /// # }
    /// ```
    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    /// Creates a new context which not holds an update.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let ctx = client.new_ctx();
    /// # }
    /// ```
    pub fn new_ctx(&self) -> Context {
        let upd_receiver = self.dispatcher.upd_sender.subscribe();

        Context::new(&self.inner_client, upd_receiver)
    }

    /// Listen to Telegram's updates and send them to the dispatcher's routers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// client.run().await?;
    /// # }
    /// ```
    pub async fn run(self) -> Result<()> {
        let handle = self.inner_client;
        let dispatcher = self.dispatcher;
        let err_handler = self.err_handler;
        let ready_handler = self.ready_handler;

        if self.set_bot_commands {
            let mut commands = Vec::new();

            let command_filters = dispatcher.get_commands();
            for command_filter in command_filters.into_iter() {
                let patterns = command_filter
                    .command
                    .split("|")
                    .filter(|pattern| pattern.len() > 1)
                    .collect::<Vec<_>>();
                let description = command_filter.description;

                for pattern in patterns.iter() {
                    commands.push(tl::enums::BotCommand::Command(tl::types::BotCommand {
                        command: pattern.to_string(),
                        description: description.to_string(),
                    }));
                }
            }

            handle
                .invoke(&tl::functions::bots::SetBotCommands {
                    scope: tl::enums::BotCommandScope::Default,
                    lang_code: "en".to_string(),
                    commands,
                })
                .await?;
        }

        let client = handle.clone();

        tokio::task::spawn(async move {
            if let Some(mut handler) = ready_handler {
                let mut injector = di::Injector::default();
                injector.insert(handle.clone());

                handler.handle(&mut injector).await.unwrap();
            }

            loop {
                match handle.next_update().await {
                    Ok(update) => {
                        let client = handle.clone();
                        let mut dp = dispatcher.clone();
                        let err_handler = err_handler.clone();

                        tokio::task::spawn(async move {
                            if let Err(e) = dp.handle_update(&client, &update).await {
                                if let Some(err_handler) = err_handler.as_ref() {
                                    err_handler.run(client, update, e).await;
                                } else {
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

            if let Some(mut handler) = self.exit_handler {
                let mut injector = di::Injector::default();
                injector.insert(client.clone());

                handler.handle(&mut injector).await.unwrap();
            }

            let session_file = self.session_file.as_deref().unwrap_or("./ferogram.session");
            client.session().save_to_file(session_file)?;
        }

        Ok(())
    }

    /// Keeps the connection open, but doesn't listen to the updates.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// client.keep_alive().await?;
    /// # }
    /// ```
    pub async fn keep_alive(self) -> Result<()> {
        let handle = self.inner_client;

        tokio::task::spawn(async move {
            loop {
                handle.step().await.unwrap();
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
    /// The client type.
    client_type: ClientType,

    /// Developer's API ID.
    api_id: i32,
    /// Developer's API hash.
    api_hash: String,
    /// The session file path.
    session_file: Option<String>,
    /// The initial parameters.
    init_params: InitParams,

    /// Whether is to update Telegram's bot commands.
    set_bot_commands: bool,
    /// Whether is to wait for a `Ctrl + C` signal to close the connection and exit the app.
    wait_for_ctrl_c: bool,

    /// The global error handler.
    pub(crate) err_handler: Option<Box<dyn ErrorHandler>>,
    /// The exit handler.
    pub(crate) exit_handler: Option<di::Endpoint>,
    /// The ready handler.
    pub(crate) ready_handler: Option<di::Endpoint>,
}

impl ClientBuilder {
    /// Creates a new builder to bot instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let client = unimplemented!();
    /// let client = Client::bot(std::env::var("BOT_TOKEN").unwrap());
    /// # }
    /// ```
    pub fn bot<T: Into<String>>(token: T) -> Self {
        Self {
            client_type: ClientType::Bot(token.into()),

            ..Default::default()
        }
    }

    /// Creates a new builder to user instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let client = unimplemented!();
    /// let client = Client::user(std::env::var("PHONE_NUMBER").unwrap());
    /// # }
    /// ```
    pub fn user<N: Into<String>>(phone_number: N) -> Self {
        Self {
            client_type: ClientType::User(phone_number.into()),

            ..Default::default()
        }
    }

    /// Builds the `Client` instance.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let client = unimplemented!();
    /// let client = client.build().await?;
    /// # }
    /// ```
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
            dispatcher: Dispatcher::default(),
            client_type: self.client_type,
            inner_client,

            session_file: Some(session_file.to_string()),

            is_connected: false,
            set_bot_commands: self.set_bot_commands,
            wait_for_ctrl_c: self.wait_for_ctrl_c,

            err_handler: self.err_handler,
            exit_handler: self.exit_handler,
            ready_handler: self.ready_handler,
        })
    }

    /// Builds and connects the `Client` instance.
    ///
    /// Connects to the Telegram server, but don't listen to updates.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let client = unimplemented!();
    /// let client = client.build_and_connect().await?;
    /// # }
    /// ```
    pub async fn build_and_connect(self) -> Result<Client> {
        self.build().await?.connect().await
    }

    /// Developer's API ID, required to interact with the Telegram's API.
    ///
    /// You may obtain your own in <https://my.telegram.org/auth>.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.api_id(123456789);
    /// # }
    /// ```
    pub fn api_id(mut self, api_id: i32) -> Self {
        self.api_id = api_id;
        self
    }

    /// Developer's API hash, required to interact with Telegram's API.
    ///
    /// You may obtain your own in <https://my.telegram.org/auth>.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.api_hash("123456789");
    /// # }
    /// ```
    pub fn api_hash<H: Into<String>>(mut self, api_hash: H) -> Self {
        self.api_hash = api_hash.into();
        self
    }

    /// Session storage where data should persist, such as authorization key, server address,
    /// and other required information by the client.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.session_file("path/to/file");
    /// # }
    /// ```
    pub fn session_file<P: AsRef<Path> + ToString>(mut self, path: P) -> Self {
        self.session_file = Some(path.to_string());
        self
    }

    /// User's device model.
    ///
    /// Telegram uses to know your device in devices settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.device_model("iPhone 14");
    /// # }
    /// ```
    pub fn device_model<M: Into<String>>(mut self, device_model: M) -> Self {
        self.init_params.device_model = device_model.into();
        self
    }

    /// User's system version.
    ///
    /// Telegram uses to know your system version in devices settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.system_version("iOS 18.2");
    /// # }
    /// ```
    pub fn system_version<V: Into<String>>(mut self, system_version: V) -> Self {
        self.init_params.system_version = system_version.into();
        self
    }

    /// Client's app version.
    ///
    /// Telegram uses to know your app version in device settings.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.app_version("1.0.0");
    /// # }
    /// ```
    pub fn app_version<V: Into<String>>(mut self, app_version: V) -> Self {
        self.init_params.app_version = app_version.into();
        self
    }

    /// Client's language code.
    ///
    /// Telegram uses internally to let others know your language.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.lang_code("en");
    /// # }
    /// ```
    pub fn lang_code<C: Into<String>>(mut self, lang_code: C) -> Self {
        self.init_params.lang_code = lang_code.into();
        self
    }

    /// Should the client catch-up on updates sent to it while it was offline?
    ///
    /// By default, updates sent while the client was offline are ignored.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.catch_up(true);
    /// # }
    /// ```
    pub fn catch_up(mut self, value: bool) -> Self {
        self.init_params.catch_up = value;
        self
    }

    /// Server address to connect to. By default, the library will connect to the address stored
    /// in the session file (or a default production address if no such address exists). This
    /// field can be used to override said address, and is most commonly used to connect to one
    /// of Telegram's test servers instead.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.server_address("127.0.0.1:8080");
    /// # }
    /// ```
    pub fn server_address(mut self, server_address: ServerAddr) -> Self {
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.flood_sleep_threshold(20);
    /// # }
    /// ```
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.update_queue_limit(Some(100));
    /// # }
    /// ```
    pub fn update_queue_limit(mut self, update_queue_limit: Option<usize>) -> Self {
        self.init_params.update_queue_limit = update_queue_limit;
        self
    }

    /// Waits for a `Ctrl + C` signal to close the connection and exit the app.
    ///
    /// Otherwise the code will continue running until it finds the end.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.wait_for_ctrl_c();
    /// # }
    /// ```
    pub fn wait_for_ctrl_c(mut self) -> Self {
        self.wait_for_ctrl_c = true;
        self
    }

    /// Updates the Telegram-side bot's command list by collecting all the commands
    /// from the dispatcher's handlers.
    ///
    /// Only commands that has more than `1` char will be registered.
    /// Ex: `start`, `help`...
    pub fn set_bot_commands(mut self) -> Self {
        self.set_bot_commands = true;
        self
    }

    /// Sets the reconnection policy.
    ///
    /// Executed when the client loses the connection or the Telegram server closes it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// use std::{ops::ControlFlow, time::Duration};
    ///
    /// use grammers_client::ReconnectionPolicy;
    ///
    /// struct MyReconnectionPolicy;
    ///
    /// impl ReconnectionPolicy for MyReconnectionPolicy {
    ///     fn should_retry(&self, attempt: usize) -> ControlFlow<(), Duration> {
    ///         if attempt < 3 {
    ///             let time = 5 * attempt;
    ///             ControlFlow::Continue(Duration::from_secs(time as u64))
    ///         } else {
    ///             ControlFlow::Break(())
    ///         }
    ///     }
    /// }
    ///
    /// let client = client.reconnection_policy(&MyReconnectionPolicy);
    /// # }
    /// ```
    pub fn reconnection_policy<P: ReconnectionPolicy>(mut self, policy: &'static P) -> Self {
        self.init_params.reconnection_policy = policy;
        self
    }

    /// Sets the global error handler.
    ///
    /// Executed when any `handler` returns an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.on_err(|_, _, error| async move {
    ///     println!("Error handling update: {:?}", error);
    /// });
    /// # }
    /// ```
    pub fn on_err<H: ErrorHandler>(mut self, handler: H) -> Self {
        self.err_handler = Some(Box::new(handler));
        self
    }

    /// Sets the exit handler.
    ///
    /// Only is called when used with `wait_for_ctrl_c` and the client is runned by `run()`.
    ///
    /// Executed when the client is about to exit.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.on_exit(|_, _| async move {
    ///     println!("Exiting...");
    ///
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn on_exit<I, H: di::Handler>(
        mut self,
        handler: impl di::IntoHandler<I, Handler = H>,
    ) -> Self {
        self.exit_handler = Some(Box::new(handler.into_handler()));
        self
    }

    /// Sets the ready handler.
    ///
    /// Only is called when the client is runned by `run()`.
    ///
    /// Executed when the client is ready to receive updates.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example(client: ferogram::Client) {
    /// let client = client.on_ready(|_, _| async move {
    ///     println!("Ready to receive updates!");
    ///
    ///     Ok(())
    /// });
    /// # }
    /// ```
    pub fn on_ready<I, H: di::Handler>(
        mut self,
        handler: impl di::IntoHandler<I, Handler = H>,
    ) -> Self {
        self.ready_handler = Some(Box::new(handler.into_handler()));
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
        let client = Client::bot(std::env::var("BOT_TOKEN").unwrap_or_default())
            .api_id(
                std::env::var("API_ID")
                    .unwrap_or("123456789".to_string())
                    .parse::<i32>()
                    .unwrap(),
            )
            .api_hash(std::env::var("API_HASH").unwrap_or_default())
            .build()
            .await;

        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_user() {
        let client = Client::user(std::env::var("PHONE_NUMBER").unwrap_or_default())
            .api_id(
                std::env::var("API_ID")
                    .unwrap_or("123456789".to_string())
                    .parse::<i32>()
                    .unwrap(),
            )
            .api_hash(std::env::var("API_HASH").unwrap_or_default())
            .build()
            .await;

        assert!(client.is_ok());
    }
}
