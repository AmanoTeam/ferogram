// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Handler module.

use grammers_client::{Client, Update};

use crate::{di, flow, ErrorHandler, Filter, Flow};

/// A handler.
///
/// Stores a [`Filter`], an [`di::Endpoint`] and an [`ErrorHandler`].
pub struct Handler {
    update_type: UpdateType,

    filter: Option<Box<dyn Filter>>,
    pub(crate) endpoint: Option<di::Endpoint>,
    pub(crate) err_handler: Option<Box<dyn ErrorHandler>>,
}

impl Handler {
    /// Create a new [`HandlerType::NewMessage`] handler.
    pub fn new_message<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::NewMessage,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::Raw`] handler.
    pub fn new_update<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::Raw,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::MessageEdited`] handler.
    pub fn message_edited<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::MessageEdited,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::MessageDeleted`] handler.
    pub fn message_deleted<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::MessageDeleted,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::CallbackQuery`] handler.
    pub fn callback_query<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::CallbackQuery,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::InlineQuery`] handler.
    pub fn inline_query<F: Filter>(filter: F) -> Self {
        Self {
            update_type: UpdateType::InlineQuery,

            filter: Some(Box::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Set the [`di::Endpoint`].
    pub fn then<I, H: di::Handler>(
        mut self,
        endpoint: impl di::IntoHandler<I, Handler = H>,
    ) -> Self {
        self.endpoint = Some(Box::new(endpoint.into_handler()));
        self
    }

    /// Set the error handler.
    ///
    /// Executed when the [`di::Endpoint`] returns an error.
    /// If not set, it will execute the global error handler, if any.
    ///
    /// It can be used to try to run the [`di::Endpoint`] again,
    /// with other filters or injection ways.
    pub fn on_err<H: ErrorHandler>(mut self, handler: H) -> Self {
        self.err_handler = Some(Box::new(handler));
        self
    }

    /// Check if the update should be handled.
    pub(crate) async fn check(&mut self, client: &Client, update: &Update) -> Flow {
        if *update == self.update_type {
            if let Some(ref mut filter) = self.filter {
                return filter.check(client.clone(), update.clone()).await;
            }
        }

        flow::continue_now()
    }
}

/// Update type.
#[derive(Clone, Default, PartialEq)]
pub enum UpdateType {
    /// New message handler.
    NewMessage,
    /// Message edited handler.
    MessageEdited,
    /// Message deleted handler.
    MessageDeleted,
    /// Callback query handler.
    CallbackQuery,
    /// Inline query handler.
    InlineQuery,
    /// Raw update handler.
    #[default]
    Raw,
}

impl PartialEq<Update> for UpdateType {
    fn eq(&self, other: &Update) -> bool {
        match self {
            Self::NewMessage => matches!(other, Update::NewMessage(_)),
            Self::MessageEdited => matches!(other, Update::MessageEdited(_)),
            Self::MessageDeleted => matches!(other, Update::MessageDeleted(_)),
            Self::CallbackQuery => matches!(other, Update::CallbackQuery(_)),
            Self::InlineQuery => matches!(other, Update::InlineQuery(_)),
            Self::Raw => matches!(other, Update::Raw(_)),
        }
    }
}

impl PartialEq<UpdateType> for Update {
    fn eq(&self, other: &UpdateType) -> bool {
        other == self
    }
}

/// Create a new [`HandlerType::NewMessage`] handler.
///
/// Injects [`Option<Message>`].
pub fn new_message<F: Filter>(filter: F) -> Handler {
    Handler::new_message(filter)
}

/// Create a new [`HandlerType::Raw`] handler.
///
/// Injects [`Option<Update>`].
pub fn new_update<F: Filter>(filter: F) -> Handler {
    Handler::new_update(filter)
}

/// Create a new [`HandlerType::MessageEdited`] handler.
///
/// Injects [`Option<Message>`].
pub fn message_edited<F: Filter>(filter: F) -> Handler {
    Handler::message_edited(filter)
}

/// Create a new [`HandlerType::MessageDeleted`] handler.
///
/// Injects [`Option<MessageDeletion>`].
pub fn message_deleted<F: Filter>(filter: F) -> Handler {
    Handler::message_deleted(filter)
}

/// Create a new [`HandlerType::CallbackQuery`] handler.
///
/// Injects [`Option<CallbackQuery>`].
pub fn callback_query<F: Filter>(filter: F) -> Handler {
    Handler::callback_query(filter)
}

/// Create a new [`HandlerType::InlineQuery`] handler.
///
/// Injects [`Option<InlineQuery>`].
pub fn inline_query<F: Filter>(filter: F) -> Handler {
    Handler::inline_query(filter)
}

/// Create a new [`HandlerType::Raw`] handler.
///
/// Injects [`Option<Update>`].
pub fn then<I, H: di::Handler>(endpoint: impl di::IntoHandler<I, Handler = H>) -> Handler {
    Handler {
        update_type: UpdateType::Raw,

        filter: None,
        endpoint: Some(Box::new(endpoint.into_handler())),
        err_handler: None,
    }
}
