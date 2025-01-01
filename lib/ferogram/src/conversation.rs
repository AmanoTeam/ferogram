// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Conversation module.

use grammers_client::types::{CallbackQuery, InlineQuery, InputMessage, Message};

use crate::Context;

/// A conversation.
pub struct Conversation {
    /// The actions.
    actions: Vec<Action>,
    /// The timeout of each action.
    timeout: u64,
    /// The last response.
    last_response: Option<Response>,
}

impl Conversation {
    /// Creates a new conversation.
    pub fn new(timeout: u64) -> Self {
        Self {
            actions: Vec::new(),
            timeout,
            last_response: None,
        }
    }

    /// Returns the actions.
    pub fn actions(&self) -> &Vec<Action> {
        &self.actions
    }

    /// Adds an action.
    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }

    /// Asks a question.
    pub fn ask(mut self, question: &str) -> Self {
        self.add_action(Action::WaitReply(question.into()));
        self
    }

    /// Sends a message.
    pub fn send<M: Into<InputMessage>>(mut self, message: M) -> Self {
        self.add_action(Action::SendMessage(message.into()));
        self
    }

    /// Executes a closure with the last response.
    pub fn and_then<F: FnOnce(Option<Response>) + 'static>(mut self, f: F) -> Self {
        self.add_action(Action::AndThen(Box::new(f)));
        self
    }

    /// Returns the last response.
    pub fn get_response(&self) -> Option<&Response> {
        self.last_response.as_ref()
    }

    /// Waits a message.
    pub fn wait_message(mut self) -> Self {
        self.add_action(Action::WaitMessage);
        self
    }

    /// Waits a callback query.
    pub fn wait_callback(mut self) -> Self {
        self.add_action(Action::WaitCallback);
        self
    }

    /// Waits an inline query.
    pub fn wait_inline(mut self) -> Self {
        self.add_action(Action::WaitInline);
        self
    }

    /// Processes the conversation.
    pub async fn process(mut self, context: &Context) {
        for action in self.actions.into_iter() {
            match action {
                Action::AndThen(f) => f(self.last_response.clone()),
                Action::SendMessage(message) => {
                    context
                        .client()
                        .send_message(
                            context.chat().await.expect("Failed to get chat"),
                            message.clone(),
                        )
                        .await
                        .expect("Failed to send message");
                }
                Action::WaitMessage => {
                    let message = context
                        .wait_for_message(Some(self.timeout))
                        .await
                        .expect("Failed to get message");

                    self.last_response = Some(Response::Message(message));
                }
                Action::WaitReply(message) => {
                    let message = context
                        .wait_for_reply(message, Some(self.timeout))
                        .await
                        .expect("Failed to get reply message");

                    self.last_response = Some(Response::Message(message));
                }
                Action::WaitCallback => {
                    let callback_query = context
                        .wait_for_callback_query(Some(self.timeout))
                        .await
                        .expect("Failed to get callback query");

                    self.last_response = Some(Response::Callback(callback_query));
                }
                Action::WaitInline => {
                    let inline_query = context
                        .wait_for_inline_query(Some(self.timeout))
                        .await
                        .expect("Failed to get inline query");

                    self.last_response = Some(Response::Inline(inline_query));
                }
            }
        }
    }
}

/// An action in a conversation.
pub enum Action {
    /// Executes a closure with the last response.
    AndThen(Box<dyn FnOnce(Option<Response>)>),
    /// Sends a message.
    SendMessage(InputMessage),
    /// Waits a reply.
    WaitReply(InputMessage),
    /// Waits a message.
    WaitMessage,
    /// Waits a callback query.
    WaitCallback,
    /// Waits an inline query.
    WaitInline,
}

/// A response in a conversation.
#[derive(Clone, Debug)]
pub enum Response {
    /// A message response.
    Message(Message),
    /// A callback query response.
    Callback(CallbackQuery),
    /// An inline query response.
    Inline(InlineQuery),
}
