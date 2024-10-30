// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Handler module.

use std::sync::Arc;

use grammers_client::{Client, Update};

use crate::{di, flow, ErrorHandler, Filter, Flow};

/// A handler.
///
/// Stores a [`Filter`], an [`di::Endpoint`] and an [`ErrorHandler`].
#[derive(Clone)]
pub struct Handler {
    handler_type: HandlerType,

    filter: Option<Arc<dyn Filter>>,
    pub(crate) endpoint: Option<di::Endpoint>,
    pub(crate) err_handler: Option<Box<dyn ErrorHandler>>,
}

impl Handler {
    /// Create a new [`HandlerType::NewMessage`] handler.
    pub fn new_message<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::NewMessage,

            filter: Some(Arc::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::Raw`] handler.
    pub fn new_update<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::Raw,

            filter: Some(Arc::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::MessageEdited`] handler.
    pub fn message_edited<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::MessageEdited,

            filter: Some(Arc::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::MessageDeleted`] handler.
    pub fn message_deleted<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::MessageDeleted,

            filter: Some(Arc::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::CallbackQuery`] handler.
    pub fn callback_query<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::CallbackQuery,

            filter: Some(Arc::new(filter)),
            endpoint: None,
            err_handler: None,
        }
    }

    /// Create a new [`HandlerType::InlineQuery`] handler.
    pub fn inline_query<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::InlineQuery,

            filter: Some(Arc::new(filter)),
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
    pub(crate) async fn check(&self, client: &Client, update: &Update) -> Flow {
        if self.handler_type == HandlerType::NewMessage && matches!(update, Update::NewMessage(_))
            || self.handler_type == HandlerType::MessageEdited
                && matches!(update, Update::MessageEdited(_))
            || self.handler_type == HandlerType::MessageDeleted
                && matches!(update, Update::MessageDeleted(_))
            || self.handler_type == HandlerType::CallbackQuery
                && matches!(update, Update::CallbackQuery(_))
            || self.handler_type == HandlerType::InlineQuery
                && matches!(update, Update::InlineQuery(_))
            || self.handler_type == HandlerType::Raw && matches!(update, Update::Raw(_))
        {
            if let Some(filter) = &self.filter {
                return filter.check(client.clone(), update.clone()).await;
            }
        }

        flow::continue_now()
    }
}

/// Handler type.
#[derive(Clone, Default, PartialEq)]
pub enum HandlerType {
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

/// Create a new [`HandlerType::NewMessage`] handler.
pub fn new_message<F: Filter>(filter: F) -> Handler {
    Handler::new_message(filter)
}

/// Create a new [`HandlerType::Raw`] handler.
pub fn new_update<F: Filter>(filter: F) -> Handler {
    Handler::new_update(filter)
}

/// Create a new [`HandlerType::MessageEdited`] handler.
pub fn message_edited<F: Filter>(filter: F) -> Handler {
    Handler::message_edited(filter)
}

/// Create a new [`HandlerType::MessageDeleted`] handler.
pub fn message_deleted<F: Filter>(filter: F) -> Handler {
    Handler::message_deleted(filter)
}

/// Create a new [`HandlerType::CallbackQuery`] handler.
pub fn callback_query<F: Filter>(filter: F) -> Handler {
    Handler::callback_query(filter)
}

/// Create a new [`HandlerType::InlineQuery`] handler.
pub fn inline_query<F: Filter>(filter: F) -> Handler {
    Handler::inline_query(filter)
}

/// Create a new [`HandlerType::Raw`] handler.
pub fn then<I, H: di::Handler>(endpoint: impl di::IntoHandler<I, Handler = H>) -> Handler {
    Handler {
        handler_type: HandlerType::Raw,

        filter: None,
        endpoint: Some(Box::new(endpoint.into_handler())),
        err_handler: None,
    }
}
