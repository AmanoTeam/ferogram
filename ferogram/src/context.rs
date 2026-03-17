// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Update context traits and methods.

use std::{ops::Deref, sync::Arc, time::Duration};

use grammers::{
    Client, InvocationError,
    media::{Media, Photo},
    message::{InputMessage, Message},
    peer::{ActionSender, Peer},
    update::{CallbackQuery, InlineQuery, InlineSend, Update},
};
use grammers_session::types::PeerRef;
use tokio::{
    sync::{Mutex, broadcast::Receiver},
    time::sleep,
};

use crate::filter::Filter;

pub struct Context {
    /// Telegram's wrapper client.
    pub client: Client,
    /// Context's main update.
    pub update: Update,
    /// Update receiver from dispatcher.
    update_rx: Arc<Mutex<Receiver<Update>>>,
}

impl Context {
    /// Create a new update context.
    pub(crate) fn new(client: Client, update: Update, update_rx: Receiver<Update>) -> Self {
        Self {
            client,
            update,
            update_rx: Arc::new(Mutex::new(update_rx)),
        }
    }

    /// The peer where this update was received.
    pub fn peer(&self) -> Option<Peer> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.peer().cloned(),
            Update::CallbackQuery(query) => query.peer().cloned(),
            _ => None,
        }
    }

    /// Cached reference to [`Self::peer`], if it is in cache.
    pub async fn peer_ref(&self) -> Option<PeerRef> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.peer_ref().await
            }
            Update::CallbackQuery(query) => query.peer_ref().await,
            _ => None,
        }
    }

    /// The peer who sent this update.
    pub fn sender(&self) -> Option<Peer> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.sender().cloned()
            }
            Update::CallbackQuery(query) => query.sender().cloned(),
            Update::InlineQuery(query) => query.sender().cloned().map(Peer::User),
            Update::InlineSend(send) => send.sender().cloned().map(Peer::User),
            _ => None,
        }
    }

    /// Cached reference to [`Self::sender`], if it is in cache.
    pub async fn sender_ref(&self) -> Option<PeerRef> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.sender_ref().await
            }
            Update::CallbackQuery(query) => query.sender_ref().await,
            Update::InlineQuery(query) => query.sender_ref().await,
            Update::InlineSend(send) => send.sender_ref().await,
            _ => None,
        }
    }

    /// Get the media attached in the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn media(&self) -> Option<Media> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.media(),
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                message.media()
            }
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Get the photo attached in the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn photo(&self) -> Option<Photo> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.photo(),
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                message.photo()
            }
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Content of the update.
    ///
    /// It differs for each update type:
    /// * [`Update::NewMessage`]: its text/caption.
    /// * [`Update::MessageEdited`]: its text/caption.
    /// * [`Update::CallbackQuery`]: its callback data.
    /// * [`Update::InlineQuery`]: its text.
    /// * [`Update::InlineSend`]: its text.
    pub fn content(&self) -> Option<String> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                Some(message.text().to_string())
            }
            Update::CallbackQuery(query) => Some(String::from_utf8_lossy(query.data()).to_string()),
            Update::InlineQuery(query) => Some(query.text().to_string()),
            Update::InlineSend(send) => Some(send.text().to_string()),
            _ => None,
        }
    }

    /// Get the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn message(&self) -> Option<Message> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                Some(message.deref().clone())
            }
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                Some(message)
            }
            _ => None,
        }
    }

    /// Get the callback query held by the update.
    pub fn callback_query(&self) -> Option<CallbackQuery> {
        match &self.update {
            Update::CallbackQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Get the inline query held by the update.
    pub fn inline_query(&self) -> Option<InlineQuery> {
        match &self.update {
            Update::InlineQuery(query) => Some(query.clone()),
            _ => None,
        }
    }

    /// Get the inline send held by the update.
    pub fn inline_send(&self) -> Option<InlineSend> {
        match &self.update {
            Update::InlineSend(send) => Some(send.clone()),
            _ => None,
        }
    }

    /// Chat action sender.
    pub async fn action(&self) -> ActionSender {
        let peer = self.peer_ref().await.unwrap();
        self.client.action(peer)
    }

    /// Check if the peer is a group.
    pub fn is_group(&self) -> bool {
        self.peer()
            .map(|peer| matches!(peer, Peer::Group(_)))
            .unwrap_or_default()
    }

    /// Check if the peer is a channel.
    pub fn is_channel(&self) -> bool {
        self.peer()
            .map(|peer| matches!(peer, Peer::Channel(_)))
            .unwrap_or_default()
    }

    /// Check if the peer is an user.
    pub fn is_private(&self) -> bool {
        self.peer()
            .map(|peer| matches!(peer, Peer::User(_)))
            .unwrap_or_default()
    }
}

/// Telegram methods.
impl Context {
    /// Edit the message held by the update.
    ///
    /// If the update is a [`Message`], it will refetch the message.
    /// If the update is a [`CallbackQuery`], it will answer the callback.
    pub async fn edit<M: Into<InputMessage>>(
        &mut self,
        new_message: M,
    ) -> Result<(), InvocationError> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.edit(new_message).await?;
                message.refetch().await
            }
            Update::CallbackQuery(query) => query.answer().edit(new_message).await,
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Send a message in reply to the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will answer the callback.
    pub async fn reply<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        match &self.update {
            Update::NewMessage(msg) | Update::MessageEdited(msg) => msg.reply(message).await,
            Update::CallbackQuery(query) => query.answer().reply(message).await,
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Delete the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn delete(&self) -> Result<(), InvocationError> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.delete().await,
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                message.delete().await
            }
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Refetch the message held by the update.
    ///
    /// Note that if you already have obtained [`Message`] through [`Self::message`],
    /// you need to call [`Message::refetch`] and not this.
    pub async fn refetch(&mut self) -> Result<(), InvocationError> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => message.refetch().await,
            _ => Ok(()),
        }
    }

    /// Send a message to the same peer, but without replying the
    /// message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will answer the callback.
    pub async fn respond<M: Into<InputMessage>>(
        &self,
        message: M,
    ) -> Result<Message, InvocationError> {
        match &self.update {
            Update::NewMessage(msg) | Update::MessageEdited(msg) => msg.respond(message).await,
            Update::CallbackQuery(query) => query.answer().respond(message).await,
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Forward the message held by the update to another chat.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn forward_to<C: Into<PeerRef>>(&self, chat: C) -> Result<Message, InvocationError> {
        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.forward_to(chat).await
            }
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                message.forward_to(chat).await
            }
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Forward the message held by the update to user's saved messages.
    ///
    /// If the update is a [`CallbackQuery`], it will load the message first.
    pub async fn forward_to_self(&self) -> Result<Message, InvocationError> {
        let chat = self.client.get_me().await?.to_ref().await.unwrap();

        match &self.update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.forward_to(chat).await
            }
            Update::CallbackQuery(query) => {
                let message = query
                    .load_message()
                    .await
                    .expect("Failed to load CallbackQuery's message");

                message.forward_to(chat).await
            }
            _ => panic!("This update doesn't contain a message"),
        }
    }

    /// Edit or reply to the message held by the update.
    ///
    /// If the update is a [`CallbackQuery`], it will answer the callback.
    pub async fn edit_or_reply<M: Into<InputMessage>>(
        &mut self,
        message: M,
    ) -> Result<Message, InvocationError> {
        let Some(Peer::User(user)) = self.peer() else {
            panic!("This update doesn't have a peer");
        };
        let from_self = user.is_self();

        if from_self {
            self.edit(message).await?;

            Ok(self.message().await.unwrap())
        } else {
            self.reply(message).await
        }
    }
}

/// Ferogram methods.
impl Context {
    /// Wait for a new update.
    ///
    /// The default timeout is 30 seconds.
    pub async fn wait_for_update(&self, timeout: Option<Duration>) -> Option<Update> {
        let mut rx = self.update_rx.lock().await;

        tokio::select! {
            _ = sleep(timeout.unwrap_or_else(|| Duration::from_secs(30))) => None,
            update = rx.recv() => update.ok(),
        }
    }

    /// Wait for a filtered new update.
    ///
    /// The default timeout is 30 seconds.
    pub async fn wait_for<F: Filter>(
        &self,
        mut filter: F,
        timeout: Option<Duration>,
    ) -> Option<Update> {
        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if filter.run(&self.client, &update).await.is_proceed() {
                    return Some(update);
                }
            } else {
                return None;
            }
        }
    }

    /// Send a message to the same peer and wait for a reply to it.
    ///
    /// The default timeout is 30 seconds.
    pub async fn wait_for_reply<M: Into<InputMessage>>(
        &self,
        message: M,
        timeout: Option<Duration>,
    ) -> Result<Option<Message>, InvocationError> {
        let sent = self.reply(message).await?;

        loop {
            if let Some(update) = self.wait_for_update(timeout).await {
                if let Update::NewMessage(message) = update
                    && let Some(msg_id) = message.reply_to_message_id()
                    && msg_id == sent.id()
                {
                    return Ok(Some(message.deref().clone()));
                }
            } else {
                return Ok(None);
            }
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let update_rx = self
            .update_rx
            .try_lock()
            .expect("Failed to lock update receiver");

        Self {
            client: self.client.clone(),
            update: self.update.clone(),
            update_rx: Arc::new(Mutex::new(update_rx.resubscribe())),
        }
    }
}
