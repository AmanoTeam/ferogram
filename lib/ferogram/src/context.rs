// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Context module.

use std::sync::Arc;

use grammers_client::{
    types::{Chat, InputMessage, Message},
    InvocationError, Update,
};
use tokio::sync::Mutex;

/// The context of an update.
#[derive(Clone, Debug)]
pub struct Context {
    client: grammers_client::Client,
    update: Update,

    waiting_for_update: Arc<Mutex<bool>>,
}

impl Context {
    /// Creates a new context.
    pub fn new(client: &grammers_client::Client, update: &Update) -> Self {
        Self {
            client: client.clone(),
            update: update.clone(),

            waiting_for_update: Arc::new(Mutex::new(false)),
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
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => Some(message.chat()),
            Update::CallbackQuery(query) => {
                let message = query.load_message().await.expect("Failed to load message");

                Some(message.chat().clone())
            }
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

    /// Returns if the update is a message.
    pub fn is_message(&self) -> bool {
        matches!(
            self.update,
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

    /// Returns if the context is waiting for an update.
    pub(crate) fn is_waiting_for_update(&self) -> bool {
        *self
            .waiting_for_update
            .try_lock()
            .expect("Failed to lock waiting_for_update")
    }
}
