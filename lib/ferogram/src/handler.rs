// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use grammers_client::{Client, Update};

use crate::{Endpoint, Filter};

/// Handler.
#[derive(Clone)]
pub struct Handler {
    handler_type: HandlerType,

    filter: Option<Arc<dyn Filter>>,
    pub(crate) endpoint: Option<Arc<dyn Endpoint>>,
}

impl Handler {
    /// Create a new `NewMessage` handler.
    pub fn new_message<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::NewMessage,

            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Create a new `Raw` handler.
    pub fn new_update<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::Raw,
            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Create a new `MessageEdited` handler.
    pub fn message_edited<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::MessageEdited,

            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Create a new `MessageDeleted` handler.
    pub fn message_deleted<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::MessageDeleted,

            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Create a new `CallbackQuery` handler.
    pub fn callback_query<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::CallbackQuery,

            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Create a new `InlineQuery` handler.
    pub fn inline_query<F: Filter>(filter: F) -> Self {
        Self {
            handler_type: HandlerType::InlineQuery,

            filter: Some(Arc::new(filter)),
            endpoint: None,
        }
    }

    /// Set the handler endpoint.
    pub fn then<E: Endpoint>(mut self, endpoint: E) -> Self {
        self.endpoint = Some(Arc::new(endpoint));
        self
    }

    /// Get the handler type.
    pub fn handler_type(&self) -> HandlerType {
        self.handler_type.clone()
    }

    /// Check if the update should be handled.
    pub(crate) async fn check(&self, client: &Client, update: &Update) -> bool {
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

        true
    }
}

/// Handle type.
#[derive(Clone, Default, PartialEq)]
pub enum HandlerType {
    NewMessage,
    MessageEdited,
    MessageDeleted,
    CallbackQuery,
    InlineQuery,
    #[default]
    Raw,
}

/// Create a new `NewMessage` handler.
pub fn new_message<F: Filter>(filter: F) -> Handler {
    Handler::new_message(filter)
}

/// Create a new `Raw` handler.
pub fn new_update<F: Filter>(filter: F) -> Handler {
    Handler::new_update(filter)
}

/// Create a new `MessageEdited` handler.
pub fn message_edited<F: Filter>(filter: F) -> Handler {
    Handler::message_edited(filter)
}

/// Create a new `MessageDeleted` handler.
pub fn message_deleted<F: Filter>(filter: F) -> Handler {
    Handler::message_deleted(filter)
}

/// Create a new `CallbackQuery` handler.
pub fn callback_query<F: Filter>(filter: F) -> Handler {
    Handler::callback_query(filter)
}

/// Create a new `InlineQuery` handler.
pub fn inline_query<F: Filter>(filter: F) -> Handler {
    Handler::inline_query(filter)
}

/// Create a new `Raw` handler.
pub fn then<E: Endpoint>(endpoint: E) -> Handler {
    Handler {
        handler_type: HandlerType::Raw,

        filter: None,
        endpoint: Some(Arc::new(endpoint)),
    }
}
