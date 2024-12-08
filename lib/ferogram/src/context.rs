// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Context module.

use std::{pin::pin, sync::Arc, time::Duration};

use futures_util::future::{select, Either};
use grammers_client::{
    types::{
        ActionSender, CallbackQuery, Chat, InlineQuery, InlineSend, InputMessage, Message,
        PackedChat, User,
    },
    InvocationError, Update,
};
use tokio::sync::{broadcast::Receiver, Mutex};

use crate::utils::bytes_to_string;

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
    /// This is useful for creating a new context with a new update.
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
    pub fn client(&self) -> &grammers_client::Client {
        &self.client
    }

    /// Returns the update.
    pub fn update(&self) -> Option<&Update> {
        self.update.as_ref()
    }

    /// Returns the chat.
    pub fn chat(&self) -> Option<Chat> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => Some(message.chat()),
            Update::CallbackQuery(query) => Some(query.chat().clone()),
            _ => None,
        }
    }

    /// Returns the text of the message.
    pub fn text(&self) -> Option<String> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                Some(message.text().to_string())
            }
            _ => None,
        }
    }

    /// Returns the sender.
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
    pub fn callback_query(&self) -> Option<CallbackQuery> {
        match self.update.as_ref().expect("No update") {
            Update::CallbackQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Returns the inline query.
    pub fn inline_query(&self) -> Option<InlineQuery> {
        match self.update.as_ref().expect("No update") {
            Update::InlineQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Returns the inline send.
    pub fn inline_send(&self) -> Option<InlineSend> {
        match self.update.as_ref().expect("No update") {
            Update::InlineSend(inline_send) => Some(inline_send.clone()),
            _ => None,
        }
    }

    /// Tries to edit the message held by the update.
    ///
    /// If the message is from the client, it will be edited.
    pub async fn edit<M: Into<InputMessage>>(&self, message: M) -> Result<(), InvocationError> {
        if let Some(msg) = self.message().await {
            msg.edit(message).await
        } else {
            panic!("Cannot edit this message")
        }
    }

    /// Tries to send a message to the chat.
    pub async fn send<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        self.client
            .send_message(self.chat().expect("No chat"), message)
            .await
    }

    /// Sends a message action.
    pub async fn action<C: Into<PackedChat>>(&self, chat: C) -> ActionSender {
        self.client.action(chat)
    }

    /// Tries to reply to the message held by the update.
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
    pub async fn delete(&self) -> Result<(), InvocationError> {
        if let Some(msg) = self.message().await {
            msg.delete().await
        } else {
            panic!("Cannot delete this message")
        }
    }

    /// Tries to refetch the message held by the update.
    ///
    /// This is useful for updating the message.
    pub async fn refetch(&self) -> Result<(), InvocationError> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.refetch().await,
            _ => panic!("Cannot refetch this message"),
        }
    }

    /// Tries to get the message that this message is replying to.
    ///
    /// If the message is not a reply, it will return `None`.
    pub async fn get_reply(&self) -> Result<Option<Message>, InvocationError> {
        if let Some(msg) = self.message().await {
            msg.get_reply().await
        } else {
            panic!("Cannot get reply to this message")
        }
    }

    /// Tries to forward the message held by the update to a chat.
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

    /// Tries to forward the message held by the update to the client's saved messages.
    ///
    /// This is useful for saving messages.
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
    pub async fn edit_or_reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        if let Some(msg) = self.message().await {
            if let Some(sender) = msg.sender() {
                if let Chat::User(user) = sender {
                    if user.is_self() {
                        msg.edit(message).await?;
                        self.refetch().await?;

                        return Ok(msg);
                    }
                }
            }

            return msg.reply(message).await;
        } else {
            panic!("Cannot edit or reply to this message")
        }
    }

    /// Tries to delete the message with the given ID.
    pub async fn delete_message(&self, message_id: i32) -> Result<(), InvocationError> {
        self.delete_messages(vec![message_id]).await.map(drop)
    }

    /// Tries to delete the messages with the given IDs.
    ///
    /// Returns the number of messages deleted.
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
    pub async fn get_message(&self, message_id: i32) -> Result<Option<Message>, InvocationError> {
        self.get_messages(vec![message_id])
            .await
            .map(|mut v| v.pop().expect("No message"))
    }

    /// Returns the messages in the chat with the given IDs.
    ///
    /// If a message is not found, it will be ignored.
    ///
    /// Not works with bot clients.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
    /* pub async fn wait_for<F: Filter>(
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
    } */

    /// Sends a message and waits for a reply to it.
    ///
    /// If the timeout is `None`, it will wait for 30 seconds.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
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
    /// If the timeout is `None`, it will wait for 30 seconds.
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
