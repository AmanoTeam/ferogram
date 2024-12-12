// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Context module.

use std::{io, path::Path, pin::pin, sync::Arc, time::Duration};

use futures_util::future::{select, Either};
use grammers_client::{
    types::{
        media::Uploaded, ActionSender, CallbackQuery, Chat, InlineQuery, InlineSend, InputMessage,
        Message, PackedChat, User,
    },
    InvocationError, Update,
};
use tokio::{
    io::AsyncRead,
    sync::{broadcast::Receiver, Mutex},
};

use crate::{utils::bytes_to_string, Filter};

/// The context of an update.
pub struct Context {
    /// The client.
    client: grammers_client::Client,
    /// The update.
    update: Option<Update>,
    /// The update receiver.
    upd_receiver: Arc<Mutex<Receiver<Update>>>,
}

impl Context {
    /// Creates a new context.
    pub fn new(client: &grammers_client::Client, upd_receiver: Receiver<Update>) -> Self {
        Self {
            client: client.clone(),
            update: None,
            upd_receiver: Arc::new(Mutex::new(upd_receiver)),
        }
    }

    /// Creates a new context with an update.
    pub fn with(
        client: &grammers_client::Client,
        update: &Update,
        upd_receiver: Receiver<Update>,
    ) -> Self {
        Self {
            client: client.clone(),
            update: Some(update.clone()),
            upd_receiver: Arc::new(Mutex::new(upd_receiver)),
        }
    }

    /// Clones the context with a new update.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let update = ctx.wait_for_update().await.unwrap();
    /// let new_ctx = ctx.clone_with(&update);
    /// # }
    /// ```
    pub fn clone_with(&self, update: &Update) -> Self {
        let upd_receiver = self
            .upd_receiver
            .try_lock()
            .expect("Failed to lock receiver");

        Self {
            client: self.client.clone(),
            update: Some(update.clone()),
            upd_receiver: Arc::new(Mutex::new(upd_receiver.resubscribe())),
        }
    }

    /// Returns the client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let client = ctx.client();
    /// # }
    /// ```
    pub fn client(&self) -> &grammers_client::Client {
        &self.client
    }

    /// Returns the update.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let update = ctx.update();
    /// # }
    /// ```
    pub fn update(&self) -> Option<&Update> {
        self.update.as_ref()
    }

    /// Returns the chat.
    ///
    /// Returns `None` if the update is not/not from a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let chat = ctx.chat();
    /// # }
    /// ```
    pub fn chat(&self) -> Option<Chat> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => Some(message.chat()),
            Update::CallbackQuery(query) => Some(query.chat().clone()),
            _ => None,
        }
    }

    /// Returns the text of the message.
    ///
    /// Returns `None` if the update is not/not from a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let text = ctx.text();
    /// # }
    /// ```
    pub fn text(&self) -> Option<String> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                Some(message.text().to_string())
            }
            _ => None,
        }
    }

    /// Returns the sender.
    ///
    /// Returns `None` if the update not has a sender.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let sender = ctx.sender();
    /// # }
    /// ```
    pub fn sender(&self) -> Option<Chat> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                Some(message.sender().expect("No sender"))
            }
            Update::CallbackQuery(query) => Some(query.sender().clone()),
            Update::InlineQuery(query) => Some(Chat::User(query.sender().clone())),
            Update::InlineSend(inline_send) => Some(Chat::User(inline_send.sender().clone())),
            _ => None,
        }
    }

    /// Returns the data of the update.
    ///
    /// Returns `None` if the update is not/not from a callback query or inline query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let query = ctx.query();
    /// # }
    /// ```
    pub fn query(&self) -> Option<String> {
        match self.update.as_ref().expect("No update") {
            Update::CallbackQuery(query) => Some(bytes_to_string(query.data())),
            Update::InlineQuery(query) => Some(query.text().to_string()),
            Update::InlineSend(inline_send) => Some(inline_send.text().to_string()),
            _ => None,
        }
    }

    /// Returns the message held by the update.
    ///
    /// If the update is a callback query, it will load the message.
    ///
    /// Returns `None` if the update is not/not from a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let message = ctx.message().await;
    /// # }
    /// ```
    pub async fn message(&self) -> Option<Message> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => Some(message.clone()),
            Update::CallbackQuery(query) => {
                let message = query.load_message().await.expect("Failed to load message");

                Some(message)
            }
            _ => None,
        }
    }

    /// Returns the callback query.
    ///
    /// Returns `None` if the update is not a callback query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let query = ctx.callback_query();
    /// # }
    /// ```
    pub fn callback_query(&self) -> Option<CallbackQuery> {
        match self.update.as_ref().expect("No update") {
            Update::CallbackQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Returns the inline query.
    ///
    /// Returns `None` if the update is not an inline query.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let query = ctx.inline_query();
    /// # }
    /// ```
    pub fn inline_query(&self) -> Option<InlineQuery> {
        match self.update.as_ref().expect("No update") {
            Update::InlineQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Returns the inline send.
    ///
    /// Returns `None` if the update is not an inline send.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let inline_send = ctx.inline_send();
    /// # }
    /// ```
    pub fn inline_send(&self) -> Option<InlineSend> {
        match self.update.as_ref().expect("No update") {
            Update::InlineSend(inline_send) => Some(inline_send.clone()),
            _ => None,
        }
    }

    /// Tries to edit the message held by the update.
    ///
    /// If the message is from the client, it will be edited.
    ///
    /// Returns `Ok(())` if the message was edited.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.edit("Hello, world!").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be edited.
    pub async fn edit<M: Into<InputMessage>>(&self, message: M) -> Result<(), InvocationError> {
        if let Some(msg) = self.message().await {
            msg.edit(message).await
        } else {
            panic!("Cannot edit this message")
        }
    }

    /// Tries to send a message to the chat.
    ///
    /// If the chat is not found, it will panic.
    ///
    /// Returns the sent message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.send("Hello, world!").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent.
    pub async fn send<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            msg.respond(message).await
        } else {
            self.client
                .send_message(self.chat().expect("No chat"), message)
                .await
        }
    }

    /// Sends a message action.
    ///
    /// Returns the action sender.
    pub async fn action<C: Into<PackedChat>>(&self, chat: C) -> ActionSender {
        self.client.action(chat)
    }

    /// Tries to reply to the message held by the update.
    ///
    /// Returns the replied message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.reply("Hello, world!").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be replied.
    pub async fn reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            msg.reply(message).await
        } else {
            panic!("Cannot reply to this message")
        }
    }

    /// Tries to delete the message held by the update.
    ///
    /// If the message is from the client, it will be deleted.
    ///
    /// Returns `Ok(())` if the message was deleted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.delete().await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be deleted.
    pub async fn delete(&self) -> Result<(), InvocationError> {
        if let Some(msg) = self.message().await {
            msg.delete().await
        } else {
            panic!("Cannot delete this message")
        }
    }

    /// Tries to refetch the message held by the update.
    ///
    /// Returns `Ok(())` if the message was refetched.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.edit("Hello, world!").await?;
    /// ctx.refetch().await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be refetched.
    pub async fn refetch(&self) -> Result<(), InvocationError> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.refetch().await,
            _ => panic!("Cannot refetch this message"),
        }
    }

    /// Tries to get the message that this message is replying to.
    ///
    /// Returns `None` if the message is not replying to another message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let reply = ctx.get_reply().await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the reply message could not be retrieved.
    pub async fn get_reply(&self) -> Result<Option<Message>, InvocationError> {
        if let Some(msg) = self.message().await {
            msg.get_reply().await
        } else {
            panic!("Cannot get reply to this message")
        }
    }

    /// Tries to forward the message held by the update to a chat.
    ///
    /// Returns the forwarded message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let chat = ctx.chat().unwrap();
    /// ctx.forward_to(chat).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be forwarded.
    pub async fn forward_to<C: Into<PackedChat>>(
        &self,
        chat: C,
    ) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            msg.forward_to(chat).await
        } else {
            panic!("Cannot forward this message")
        }
    }

    /// Tries to upload a local file to the telegram without sending it to a chat.
    ///
    /// Returns the uploaded file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let file = ctx.upload_file("path/to/file").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the file could not be uploaded.
    pub async fn upload_file<P: AsRef<Path>>(&self, path: P) -> Result<Uploaded, io::Error> {
        self.client.upload_file(path).await
    }

    /// Tries to upload a stream to the telegram without sending it to a chat.
    ///
    /// Returns the uploaded file.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let stream = tokio::fs::File::open("path/to/file").await?;
    /// let file = ctx.upload_stream(&mut stream, 1024, "file.txt").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the stream could not be uploaded.
    pub async fn upload_stream<S: AsyncRead + Unpin>(
        &self,
        stream: &mut S,
        size: usize,
        name: String,
    ) -> Result<Uploaded, io::Error> {
        self.client.upload_stream(stream, size, name).await
    }

    /// Tries to forward the message held by the update to the client's saved messages.
    ///
    /// Returns the forwarded message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.forward_to_self().await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be forwarded.
    pub async fn forward_to_self(&self) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            let chat = self.client().get_me().await?;

            msg.forward_to(chat).await
        } else {
            panic!("Cannot forward this message")
        }
    }

    /// Tries to edit or reply to the message held by the update.
    ///
    /// If the message is from the client, it will be edited.
    ///
    /// Returns the edited or replied message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.edit_or_reply("Hello, world!").await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be edited or replied.
    pub async fn edit_or_reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            if let Some(sender) = msg.sender() {
                if let Chat::User(user) = sender {
                    if user.is_self() {
                        msg.edit(message).await?;
                        // FIXME: uncomment when `Message::refetch` fully works
                        // self.refetch().await?;

                        return Ok(msg);
                    }
                }
            }

            return msg.reply(message).await;
        } else {
            panic!("Cannot edit or reply to this message")
        }
    }

    /// Tries to delete a message with the given ID in the chat.
    ///
    /// Returns `Ok(())` if the message was deleted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.delete_message(1234).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be deleted.
    pub async fn delete_message(&self, message_id: i32) -> Result<(), InvocationError> {
        self.delete_messages(vec![message_id]).await.map(drop)
    }

    /// Tries to delete the messages with the given IDs in the chat.
    ///
    /// Returns the number of messages deleted.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// ctx.delete_messages(vec![1234, 5678]).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the messages could not be deleted.
    pub async fn delete_messages(&self, message_ids: Vec<i32>) -> Result<usize, InvocationError> {
        self.client
            .delete_messages(self.chat().expect("No chat"), &message_ids)
            .await
    }

    /// Returns the message in the chat with the given ID.
    ///
    /// If the message is not found, it will return `None`.
    ///
    /// Not works with bot clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let message = ctx.get_message(1234).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be retrieved.
    pub async fn get_message(&self, message_id: i32) -> Result<Option<Message>, InvocationError> {
        self.get_messages(vec![message_id])
            .await
            .map(|mut v| v.pop().unwrap_or(None))
    }

    /// Returns the messages in the chat with the given IDs.
    ///
    /// If a message is not found, it will be ignored.
    ///
    /// Not works with bot clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let messages = ctx.get_messages(vec![1234, 5678]).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the messages could not be retrieved.
    pub async fn get_messages(
        &self,
        message_ids: Vec<i32>,
    ) -> Result<Vec<Option<Message>>, InvocationError> {
        self.client
            .get_messages_by_id(self.chat().expect("No chat"), &message_ids)
            .await
    }

    /// Returns the total number of messages in the chat.
    ///
    /// This may be slow for large chats.
    ///
    /// Not works with bot clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let total = ctx.total_messages().await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the total number of messages could not be retrieved.
    pub async fn total_messages(&self) -> Result<usize, InvocationError> {
        self.client
            .iter_messages(self.chat().expect("No chat"))
            .total()
            .await
    }

    /// Returns the messages in the chat from the given user.
    ///
    /// If the limit is `None`, it will be set to `100`.
    ///
    /// Not works with bot clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let Chat::User(user) = ctx.sender().unwrap();
    /// let messages = ctx.get_messages_from(&user, None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the messages could not be retrieved.
    pub async fn get_messages_from(
        &self,
        user: &User,
        limit: Option<usize>,
    ) -> Result<Vec<Message>, InvocationError> {
        let mut iter = self
            .client
            .iter_messages(self.chat().expect("No chat"))
            .limit(limit.unwrap_or(100));
        let mut messages = Vec::new();

        while let Some(message) = iter.next().await? {
            if let Some(sender) = message.sender() {
                if matches!(sender, Chat::User(u) if u.id() == user.id()) {
                    messages.push(message);
                }
            }
        }

        Ok(messages)
    }

    /// Returns the messages in the chat from the client.
    ///
    /// If the limit is `None`, it will be set to `100`.
    ///
    /// Not works with bot clients.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let messages = ctx.get_messages_from_self(None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the messages could not be retrieved.
    pub async fn get_messages_from_self(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<Message>, InvocationError> {
        let mut iter = self
            .client
            .iter_messages(self.chat().expect("No chat"))
            .limit(limit.unwrap_or(100));
        let mut messages = Vec::new();

        while let Some(message) = iter.next().await? {
            if let Some(sender) = message.sender() {
                if matches!(sender, Chat::User(user) if user.is_self()) {
                    messages.push(message);
                }
            }
        }

        Ok(messages)
    }

    /// Waits for an update.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// Returns `None` if the timeout is reached.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let update = ctx.wait_for_update(None).await?;
    /// # }
    /// ```
    pub async fn wait_for_update(&self, timeout: Option<u64>) -> Option<Update> {
        let mut rx = self.upd_receiver.lock().await;

        loop {
            let stop = pin!(async {
                tokio::time::sleep(Duration::from_secs(timeout.unwrap_or(30))).await
            });
            let upd = pin!(async { rx.recv().await });

            match select(stop, upd).await {
                Either::Left(_) => return None,
                Either::Right((update, _)) => return update.ok(),
            }
        }
    }

    /// Waits for an update that matches the filter.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// use ferogram::flow;
    /// use grammers_client::Update;
    ///
    /// let update = ctx.wait_for(|_, update| async move {
    ///     if let Update::NewMessage(message) = update {
    ///         if message.text() == "Hello, world!" {
    ///             return flow::continue_now();
    ///         }
    ///     }
    ///
    ///     flow::break_now()
    /// }, None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the update could not be received.
    pub async fn wait_for<F: Filter>(
        &self,
        mut filter: F,
        timeout: Option<u64>,
    ) -> Result<Update, crate::Error> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if filter
                    .check(self.client.clone(), update.clone())
                    .await
                    .is_continue()
                {
                    return Ok(update);
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Sends a message and waits for a reply to it.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let message = ctx.wait_for_reply("Hello, world!", None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be sent or the reply could not be received.
    pub async fn wait_for_reply<M: Into<InputMessage>>(
        &self,
        message: M,
        timeout: Option<u64>,
    ) -> Result<Message, crate::Error> {
        let sent = self.reply(message).await?;

        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::NewMessage(msg) | Update::MessageEdited(msg) = update {
                    if let Some(msg_id) = msg.reply_to_message_id() {
                        if msg_id == sent.id() {
                            return Ok(msg);
                        }
                    }
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Waits for a message.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let message = ctx.wait_for_message(None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the message could not be received.
    pub async fn wait_for_message(&self, timeout: Option<u64>) -> Result<Message, crate::Error> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::NewMessage(message) = update {
                    return Ok(message);
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Waits for a callback query.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let query = ctx.wait_for_callback_query(None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the callback query could not be received.
    pub async fn wait_for_callback_query(
        &self,
        timeout: Option<u64>,
    ) -> Result<CallbackQuery, crate::Error> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::CallbackQuery(query) = update {
                    return Ok(query);
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Waits for a inline query.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let query = ctx.wait_for_inline_query(None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the inline query could not be received.
    pub async fn wait_for_inline_query(
        &self,
        timeout: Option<u64>,
    ) -> Result<InlineQuery, crate::Error> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::InlineQuery(query) = update {
                    return Ok(query);
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Waits for a inline send.
    ///
    /// If the timeout is `None`, it will be set to 30 seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let ctx = unimplemented!();
    /// let inline_send = ctx.wait_for_inline_send(None).await?;
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the inline send could not be received.
    pub async fn wait_for_inline_send(
        &self,
        timeout: Option<u64>,
    ) -> Result<InlineSend, crate::Error> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::InlineSend(inline_send) = update {
                    return Ok(inline_send);
                }
            } else {
                return Err(crate::Error::timeout(timeout.unwrap()));
            }
        }
    }

    /// Returns if the chat is private (an user).
    pub fn is_private(&self) -> bool {
        self.chat()
            .map(|chat| matches!(chat, Chat::User(_)))
            .unwrap_or(false)
    }

    /// Returns if the chat is a group.
    pub fn is_group(&self) -> bool {
        self.chat()
            .map(|chat| matches!(chat, Chat::Group(_)))
            .unwrap_or(false)
    }

    /// Returns if the chat is a channel.
    pub fn is_channel(&self) -> bool {
        self.chat()
            .map(|chat| matches!(chat, Chat::Channel(_)))
            .unwrap_or(false)
    }

    /// Returns if the update is a message.
    pub fn is_message(&self) -> bool {
        matches!(
            self.update.as_ref().expect("No update"),
            Update::NewMessage(_) | Update::MessageEdited(_)
        )
    }

    /// Returns if the update is a edited message.
    pub fn is_edited(&self) -> bool {
        matches!(
            self.update.as_ref().expect("No update"),
            Update::MessageEdited(_)
        )
    }

    /// Returns if the update is a callback query.
    pub fn is_callback_query(&self) -> bool {
        matches!(
            self.update.as_ref().expect("No update"),
            Update::CallbackQuery(_)
        )
    }

    /// Returns if the update is a inline query.
    pub fn is_inline_query(&self) -> bool {
        matches!(
            self.update.as_ref().expect("No update"),
            Update::InlineQuery(_)
        )
    }

    /// Returns if the update is a inline send.
    pub fn is_inline_send(&self) -> bool {
        matches!(
            self.update.as_ref().expect("No update"),
            Update::InlineSend(_)
        )
    }

    /// Returns if is a raw update.
    pub fn is_raw(&self) -> bool {
        matches!(self.update.as_ref().expect("No update"), Update::Raw(_))
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let upd_receiver = self
            .upd_receiver
            .try_lock()
            .expect("Failed to lock receiver");

        Self {
            client: self.client.clone(),
            update: self.update.clone(),
            upd_receiver: Arc::new(Mutex::new(upd_receiver.resubscribe())),
        }
    }
}
