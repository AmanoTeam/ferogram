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
    types::{CallbackQuery, Chat, InlineQuery, InlineSend, InputMessage, Message},
    InvocationError, Update,
};
use tokio::sync::{broadcast::Receiver, Mutex};

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

    /// Returns the client.
    pub fn client(&self) -> &grammers_client::Client {
        &self.client
    }

    /// Returns the update.
    pub fn update(&self) -> Option<&Update> {
        self.update.as_ref()
    }

    /// Try to return the chat.
    pub async fn chat(&self) -> Option<Chat> {
        self.message().await.map(|msg| msg.chat())
    }

    /// Try to return the message.
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

    /// Try to return the callback query.
    pub async fn callback_query(&self) -> Option<CallbackQuery> {
        match self.update.as_ref().expect("No update") {
            Update::CallbackQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Try to return the inline query.
    pub async fn inline_query(&self) -> Option<InlineQuery> {
        match self.update.as_ref().expect("No update") {
            Update::InlineQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Try to return the inline send.
    pub async fn inline_send(&self) -> Option<InlineSend> {
        match self.update.as_ref().expect("No update") {
            Update::InlineSend(inline_send) => Some(inline_send.clone()),
            _ => None,
        }
    }

    /// Try to directly reply to the message held by the update.
    pub async fn reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(msg) | Update::MessageEdited(msg) => msg.reply(message).await,
            Update::CallbackQuery(query) => {
                let msg = query.load_message().await.expect("Failed to load message");

                msg.reply(message).await
            }
            _ => panic!("Cannot reply to this update"),
        }
    }

    /// Try to directly edit the message held by the update.
    pub async fn edit<M: Into<InputMessage>>(&self, message: M) -> Result<(), InvocationError> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(msg) | Update::MessageEdited(msg) => msg.edit(message).await,
            Update::CallbackQuery(query) => {
                let msg = query.load_message().await.expect("Failed to load message");

                msg.edit(message).await
            }
            _ => panic!("Cannot edit this update"),
        }
    }

    /// Try to directly delete the message held by the update.
    pub async fn delete(&self) -> Result<(), InvocationError> {
        match self.update.as_ref().expect("No update") {
            Update::NewMessage(msg) | Update::MessageEdited(msg) => msg.delete().await,
            Update::CallbackQuery(query) => {
                let msg = query.load_message().await.expect("Failed to load message");

                msg.delete().await
            }
            _ => panic!("Cannot delete this update"),
        }
    }

    /// Wait for an update.
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
                Either::Right((update, _)) => {
                    return Some(update.expect("Failed to receive update"))
                }
            }
        }
    }

    /// Wait for a reply to a message.
    ///
    /// If the timeout is `None`, it will wait for 30 seconds.
    pub async fn wait_for_reply<M: Into<InputMessage>>(
        &self,
        message: M,
        timeout: Option<u64>,
    ) -> Result<Message, crate::Error> {
        let sent = self.reply(message).await.expect("Failed to reply");

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

    /// Wait for a message.
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

    /// Wait for a callback query.
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

    /// Wait for a inline query.
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

    /// Wait for a inline send.
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
