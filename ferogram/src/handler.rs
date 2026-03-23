// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Routes to be dispatched with some [`crate::filter`]s.

use grammers::{Client, update::Update};

#[cfg(feature = "macros")]
pub use ferogram_macros::*;

use crate::{
    di::{self, Endpoint, Injector},
    filter::{Filter, IntoFilter},
};

/// Result type expected to be returned by an [`Endpoint`].
pub type Result = crate::Result<()>;

/// A route that stores a single or nested [`Filter`] and an [`Endpoint`].
pub struct Handler {
    /// Which type of update this handler should listen for.
    r#type: UpdateType,

    filter: Option<Box<dyn Filter>>,
    endpoint: Option<Endpoint>,
}

impl Handler {
    /// Create a `Raw` handler.
    pub fn new_raw<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::Raw,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `NewMessage` handler.
    pub fn new_message<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::NewMessage,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `MessageEdited` handler.
    pub fn message_edited<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::MessageEdited,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `MessageDeleted` handler.
    pub fn message_deleted<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::MessageDeleted,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `CallbackQuery` handler.
    pub fn callback_query<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::CallbackQuery,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `InlineQuery` handler.
    pub fn inline_query<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::InlineQuery,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Create a `InlineSend` handler.
    pub fn inline_send<Marker>(filter: impl IntoFilter<Marker>) -> Self {
        Self {
            r#type: UpdateType::InlineSend,

            filter: Some(Box::new(filter.into_filter())),
            endpoint: None,
        }
    }

    /// Set the handler's [`Endpoint`], which will be executed after
    /// the filter.
    pub fn then<I, H: di::RequestHandler>(
        mut self,
        endpoint: impl di::IntoRequestHandler<I, Handler = H>,
    ) -> Self {
        self.endpoint = Some(Box::new(endpoint.into_handler()));
        self
    }

    /// Execute the handler.
    pub(crate) async fn run(
        &mut self,
        client: &Client,
        update: &Update,
        mut injector: Injector,
    ) -> crate::Result<bool> {
        if self.r#type == *update {
            if let Some(ref mut filter) = self.filter {
                let flow = filter.run(client, update).await;
                if flow.is_stop() {
                    return Ok(false);
                }

                injector.extend(flow.injector);
            }

            if let Some(ref mut endpoint) = self.endpoint {
                match update {
                    Update::Raw(raw) => {
                        injector.push(raw.clone());
                    }
                    Update::NewMessage(message) | Update::MessageEdited(message) => {
                        injector.push(message.clone());
                    }
                    Update::MessageDeleted(message_deletion) => {
                        injector.push(message_deletion.clone());
                    }
                    Update::CallbackQuery(query) => {
                        injector.push(query.clone());
                    }
                    Update::InlineQuery(query) => {
                        injector.push(query.clone());
                    }
                    Update::InlineSend(send) => {
                        injector.push(send.clone());
                    }
                    upd => {
                        tracing::debug!("Unhandled update type: {upd:?}");
                    }
                }

                endpoint.handle(injector).await?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum UpdateType {
    NewMessage,
    MessageEdited,
    MessageDeleted,
    CallbackQuery,
    InlineQuery,
    InlineSend,
    #[default]
    Raw,
}

impl From<String> for UpdateType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "new_message" => Self::NewMessage,
            "message_edited" => Self::MessageEdited,
            "message_deleted" => Self::MessageDeleted,
            "callback_query" => Self::CallbackQuery,
            "inline_query" => Self::InlineQuery,
            "inline_send" => Self::InlineSend,
            _ => Self::default(),
        }
    }
}

impl PartialEq<Update> for UpdateType {
    fn eq(&self, other: &Update) -> bool {
        match self {
            Self::NewMessage => matches!(other, Update::NewMessage(_)),
            Self::MessageEdited => matches!(other, Update::MessageEdited(_)),
            Self::MessageDeleted => matches!(other, Update::MessageDeleted(_)),
            Self::CallbackQuery => matches!(other, Update::CallbackQuery(_)),
            Self::InlineQuery => matches!(other, Update::InlineQuery(_)),
            Self::InlineSend => matches!(other, Update::InlineSend(_)),
            Self::Raw => matches!(other, Update::Raw(_)),
        }
    }
}

impl PartialEq<UpdateType> for Update {
    fn eq(&self, other: &UpdateType) -> bool {
        other == self
    }
}

/// Creates a `Raw` handler.
///
/// Injects [`grammers::update::Raw`].
pub fn new_raw<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::new_raw(filter)
}

/// Create a `NewMessage` handler.
///
/// Injects [`grammers::update::Message`].
pub fn new_message<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::new_message(filter)
}

/// Create a `MessageEdited` handler.
///
/// Injects [`grammers::update::Message`].
pub fn message_edited<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::message_edited(filter)
}

/// Create a `MessageDeleted` handler.
///
/// Injects [`grammers::update::MessageDeletion`].
pub fn message_deleted<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::message_deleted(filter)
}

/// Create a `CallbackQuery` handler.
///
/// Injects [`grammers::update::CallbackQuery`].
pub fn callback_query<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::callback_query(filter)
}

/// Create a `InlineQuery` handler.
///
/// Injects [`grammers::update::InlineQuery`].
pub fn inline_query<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::inline_query(filter)
}

/// Create a `InlineSend` handler.
///
/// Injects [`grammers::update::InlineSend`].
pub fn inline_send<Marker>(filter: impl IntoFilter<Marker>) -> Handler {
    Handler::inline_send(filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_handler() {
        new_raw(move |_: Client, _: Update| Ok(()));
        message_deleted(move |_: Client, _: Update| Ok(()));
        message_edited(move |_: Client, _: Update| Ok(()));
        callback_query(move |_: Client, _: Update| Ok(()));
        inline_query(move |_: Client, _: Update| Ok(()));
        inline_send(move |_: Client, _: Update| Ok(()));
    }

    #[test]
    fn test_async_handler() {
        new_raw(|_: Client, _: Update| async move { Ok(()) });
        message_deleted(|_: Client, _: Update| async move { Ok(()) });
        message_edited(|_: Client, _: Update| async move { Ok(()) });
        callback_query(|_: Client, _: Update| async move { Ok(()) });
        inline_query(|_: Client, _: Update| async move { Ok(()) });
        inline_send(|_: Client, _: Update| async move { Ok(()) });
    }
}
