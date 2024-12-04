// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Context module.

use std::{pin::pin, time::Duration};

use futures_util::future::{select, Either};
use grammers_client::{
    types::{CallbackQuery, Chat, InlineQuery, InlineSend, InputMessage, Message},
    InvocationError, Update,
};
use tokio::sync::broadcast::Receiver;

/// The context of an update.
pub struct Context {
    /// The client.
    client: grammers_client::Client,
    /// The update.
    update: Update,
    /// The update receiver.
    upd_receiver: Receiver<Update>,
}

impl Context {
    /// Creates a new context.
    pub fn new(
        client: &grammers_client::Client,
        update: &Update,
        upd_receiver: Receiver<Update>,
    ) -> Self {
        Self {
            client: client.clone(),
            update: update.clone(),
            upd_receiver,
        }
    }

    /// Returns the client.
    pub fn client(&self) -> &grammers_client::Client {
        &self.client
    }

    /// Returns the update.
    pub fn update(&self) -> &Update {
        &self.update
    }

    /// Try to return the chat.
    pub async fn chat(&self) -> Option<Chat> {
        self.message().await.map(|msg| msg.chat())
    }

    /// Try to return the message.
    pub async fn message(&self) -> Option<Message> {
        match &self.update {
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
        match &self.update {
            Update::CallbackQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Try to return the inline query.
    pub async fn inline_query(&self) -> Option<InlineQuery> {
        match &self.update {
            Update::InlineQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Try to return the inline send.
    pub async fn inline_send(&self) -> Option<InlineSend> {
        match &self.update {
            Update::InlineSend(inline_send) => Some(inline_send.clone()),
            _ => None,
        }
    }

    /// Try to directly reply to the message held by the update.
    pub async fn reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        match &self.update {
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
        match &self.update {
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
        match &self.update {
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
    pub async fn wait_for_update(&mut self, timeout: Option<u64>) -> Option<Update> {
        let rx = &mut self.upd_receiver;

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
        &mut self,
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
    pub async fn wait_for_message(&mut self, timeout: Option<u64>) -> Option<Message> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::NewMessage(message) = update {
                    return Some(message);
                }
            } else {
                return None;
            }
        }
    }

    /// Wait for a callback query.
    ///
    /// If the timeout is `None`, it will wait for 30 seconds.
    pub async fn wait_for_callback_query(&mut self, timeout: Option<u64>) -> Option<CallbackQuery> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::CallbackQuery(query) = update {
                    return Some(query);
                }
            } else {
                return None;
            }
        }
    }

    /// Wait for a inline query.
    ///
    /// If the timeout is `None`, it will wait for 30 seconds.
    pub async fn wait_for_inline_query(&mut self, timeout: Option<u64>) -> Option<InlineQuery> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::InlineQuery(query) = update {
                    return Some(query);
                }
            } else {
                return None;
            }
        }
    }

    /// Wait for a inline send.
    ///
    /// If the timeout is `None`, it will wait for 30 seconds.
    pub async fn wait_for_inline_send(&mut self, timeout: Option<u64>) -> Option<InlineSend> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::InlineSend(inline_send) = update {
                    return Some(inline_send);
                }
            } else {
                return None;
            }
        }
    }

    /// Returns if the update is a message.
    pub fn is_message(&self) -> bool {
        matches!(
            self.update(),
            Update::NewMessage(_) | Update::MessageEdited(_)
        )
    }

    /// Returns if the update is a edited message.
    pub fn is_edited(&self) -> bool {
        matches!(self.update, Update::MessageEdited(_))
    }

    /// Returns if the update is a callback query.
    pub fn is_callback_query(&self) -> bool {
        matches!(self.update, Update::CallbackQuery(_))
    }

    /// Returns if the update is a inline query.
    pub fn is_inline_query(&self) -> bool {
        matches!(self.update, Update::InlineQuery(_))
    }

    /// Returns if the update is a inline send.
    pub fn is_inline_send(&self) -> bool {
        matches!(self.update, Update::InlineSend(_))
    }

    /// Returns if is a raw update.
    pub fn is_raw(&self) -> bool {
        matches!(self.update, Update::Raw(_))
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            update: self.update.clone(),
            upd_receiver: self.upd_receiver.resubscribe(),
        }
    }
}
