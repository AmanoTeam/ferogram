// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Default and useful filters.

use grammers::{Client, media::Media, peer::Peer, tl, update::Update};

use super::{AsyncMarker, FilterExt, Flow, IntoFilter, SyncMarker};

/// Always passing filter, it doesn't check for anything.
pub fn always(_: Client, _: Update) -> bool {
    true
}

/// Never passing filter, it doesn't check for anything.
pub fn never(_: Client, _: Update) -> bool {
    false
}

/// Pass if `first` or `other` pass.
pub fn or<M1, M2>(
    first: impl IntoFilter<M1>,
    other: impl IntoFilter<M2>,
) -> impl IntoFilter<AsyncMarker> {
    first.or(other)
}

/// Pass if `first` and `second` pass.
pub fn and<M1, M2>(
    first: impl IntoFilter<M1>,
    second: impl IntoFilter<M2>,
) -> impl IntoFilter<AsyncMarker> {
    first.and(second)
}

/// Pass if `filter` don't pass.
pub fn not<Marker>(filter: impl IntoFilter<Marker>) -> impl IntoFilter<AsyncMarker> {
    filter.not()
}

/// Pass if the message text contains the specified pattern.
pub fn text(pat: &'static str) -> impl IntoFilter<SyncMarker> {
    move |_, update| match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            message.text().contains(pat)
        }
        _ => false,
    }
}

/// Pass if the message text or query data matches the specified pattern.
pub fn regex(pat: &'static str) -> impl IntoFilter<SyncMarker> {
    move |_, update| match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let re = regex::Regex::new(pat).unwrap();

            re.is_match(message.text())
        }
        Update::CallbackQuery(query) => {
            let re = regex::bytes::Regex::new(pat).unwrap();

            re.is_match(query.data())
        }
        Update::InlineQuery(query) => {
            let re = regex::Regex::new(pat).unwrap();

            re.is_match(query.text())
        }
        _ => false,
    }
}

/// Pass if the message has a URL.
///
/// It extracts URLs directly from message's format entities, [`tl::enums::MessageEntity::Url`].
///
/// Note: if the `url` feature is enabled, it will also extracts URLs that weren't converted to format
/// entities, which can happen when using a broken client.
///
/// Injects:
/// * `Vec<String>`: extracted urls.
pub fn has_url(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let text = message.text();
            let mut urls = Vec::new();

            if let Some(entities) = message.fmt_entities().cloned() {
                for entity in entities
                    .into_iter()
                    .filter(|entity| matches!(entity, tl::enums::MessageEntity::Url(_)))
                {
                    let url = text
                        .chars()
                        .skip(entity.offset() as usize)
                        .take(entity.length() as usize)
                        .collect::<String>();
                    urls.push(url);
                }
            }

            #[cfg(feature = "url")]
            {
                use url::Url;

                for part in text.split_whitespace() {
                    if let Ok(url) = Url::parse(part) {
                        let url = url.to_string();

                        if !urls.contains(&url) {
                            urls.push(url);
                        }
                    }
                }
            }

            if urls.is_empty() {
                super::stop()
            } else {
                super::proceed_with(urls)
            }
        }
        _ => super::stop(),
    }
}

/// Pass if the message has a dice attached to it.
///
/// Injects:
/// * [`grammers::message::Dice`][]: dice.
pub fn has_dice(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Dice(dice)) = message.media() {
                return super::proceed_with(dice);
            }

            super::proceed()
        }
        _ => super::stop(),
    }
}

/// Pass if the update is from self.
pub fn from_self(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let Some(Peer::User(user)) = message.sender() else {
                return false;
            };

            user.is_self()
        }
        Update::CallbackQuery(query) => {
            let Some(Peer::User(user)) = query.sender() else {
                return false;
            };

            user.is_self()
        }
        Update::InlineQuery(query) => {
            let Some(user) = query.sender() else {
                return false;
            };

            user.is_self()
        }
        Update::InlineSend(query) => {
            let Some(user) = query.sender() else {
                return false;
            };

            user.is_self()
        }
        _ => false,
    }
}
