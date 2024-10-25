// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod and;
mod not;
mod or;

use std::sync::Arc;

pub(crate) use and::And;
use grammers_client::{grammers_tl_types as tl, types::Media, Client, Update};
pub(crate) use not::Not;
pub(crate) use or::Or;

use crate::Filter;

/// Always pass.
pub async fn always(_: Client, _: Update) -> bool {
    true
}

/// Never pass.
pub async fn never(_: Client, _: Update) -> bool {
    false
}

/// Pass if `first` or `other` pass.
pub fn or<F: Filter, O: Filter>(first: F, other: O) -> impl Filter {
    Or {
        first: Arc::new(first),
        other: Arc::new(other),
    }
}

/// Pass if `first` and `second` pass.
pub fn and<F: Filter, S: Filter>(first: F, second: S) -> impl Filter {
    And {
        first: Arc::new(first),
        second: Arc::new(second),
    }
}

/// Pass if `filter` don't pass.
pub fn not<F: Filter>(filter: F) -> impl Filter {
    Not {
        filter: Arc::new(filter),
    }
}

/// Pass if the message contains text.
pub fn text(pat: &'static str) -> impl Filter {
    Arc::new(move |_client, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                message.text().contains(pat)
            }
            _ => false,
        }
    })
}

/// Pass if the message has a poll.
pub async fn poll(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            matches!(message.media(), Some(Media::Poll(_)))
        }
        _ => false,
    }
}

/// Pass if the message has an audio.
pub async fn audio(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(media) = message.media() {
                if let Media::Document(document) = media {
                    return document.audio_title().is_some()
                        || document.performer().is_some()
                        || document
                            .mime_type()
                            .map_or(false, |mime| mime.starts_with("audio/"));
                }
            }

            false
        }
        _ => false,
    }
}

/// Pass if the message has a photo.
pub async fn photo(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            message.photo().is_some() || matches!(message.media(), Some(Media::Photo(_)))
        }
        _ => false,
    }
}

/// Pass if the message has a video.
pub async fn video(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(media) = message.media() {
                if let Media::Document(document) = media {
                    return document
                        .mime_type()
                        .map_or(false, |mime| mime.starts_with("video/"));
                }
            }

            false
        }
        _ => false,
    }
}

/// Pass if the message has a document.
pub async fn document(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            matches!(message.media(), Some(Media::Document(_)))
        }
        _ => false,
    }
}

/// Pass if the message has a sticker.
pub async fn sticker(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            matches!(message.media(), Some(Media::Sticker(_)))
        }
        _ => false,
    }
}

/// Pass if the message has an animated sticker.
pub async fn animated_sticker(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(media) = message.media() {
                if let Media::Document(document) = media {
                    return document.is_animated();
                }
            }

            false
        }
        _ => false,
    }
}

/// Pass if the messaage has a dice.
pub async fn dice(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            matches!(message.media(), Some(Media::Dice(_)))
        }
        _ => false,
    }
}

/// Pass if the update is a new chat member.
pub async fn new_chat_member(_: Client, update: Update) -> bool {
    if let Update::Raw(raw_update) = update {
        return matches!(raw_update, tl::enums::Update::ChatParticipantAdd(_));
    }

    false
}

/// Pass if the update is a left chat member.
pub async fn left_chat_member(_: Client, update: Update) -> bool {
    if let Update::Raw(raw_update) = update {
        return matches!(raw_update, tl::enums::Update::ChatParticipantDelete(_));
    }

    false
}

/// Pass if the update is a typing action.
pub async fn typing(_: Client, update: Update) -> bool {
    if let Update::Raw(raw_update) = update {
        return matches!(
            raw_update,
            tl::enums::Update::UserTyping(_) | tl::enums::Update::ChatUserTyping(_)
        );
    }
    false
}
