// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Default and useful filters.

use std::collections::HashMap;

use grammers::{Client, media::Media, peer::Peer, tl, update::Update};

use super::{AsyncMarker, FilterExt, Flow, IntoFilter, SyncMarker};
use crate::router::CommandParams;

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
pub fn text(pattern: &'static str) -> impl IntoFilter<SyncMarker> {
    move |_, update| match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            message.text().contains(pattern)
        }
        _ => false,
    }
}

/// Pass if the message text or query data matches the specified pattern.
pub fn regex(pattern: &'static str) -> impl IntoFilter<SyncMarker> {
    move |_, update| match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let re = regex::Regex::new(pattern).unwrap();
            re.is_match(message.text())
        }
        Update::CallbackQuery(query) => {
            let re = regex::bytes::Regex::new(pattern).unwrap();
            re.is_match(query.data())
        }
        Update::InlineQuery(query) => {
            let re = regex::Regex::new(pattern).unwrap();
            re.is_match(query.text())
        }
        _ => false,
    }
}

/// Pass if the message text or query data matches the specified command pattern.
///
/// It supports parameters, which are:
/// - `:param`: a one-word required parameter.
/// - `:param?`: a one-word optional parameter.
/// - `*param`: a multiple-word required parameter.
/// - `*param?`: a multiple-word optional parameter.
///
/// Note: optional parameters (those ending in `?`) can only be added at the
/// end of patterns, which also means that required parameters cannot follow
/// optional parameters, otherwise it'll panic.
///
/// # Injects:
/// * [`CommandParams`]: extracted params.
///
/// # Examples
///
/// ```
/// use ferogram::prelude::*;
/// handler::new_message(filter::command("/profile :id?"))
/// ```
pub fn command(pattern: &'static str) -> impl IntoFilter<SyncMarker> {
    let parts = pattern.split_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        panic!("Invalid pattern '{pattern}': it needs to have at least one word");
    }

    let mut pat = format!("^{}", regex::escape(parts[0]));

    let mut seen_multiple = false;
    let mut seen_optional = false;

    for part in &parts[1..] {
        let is_optional = part.ends_with('?');

        if seen_multiple {
            panic!(
                "Invalid pattern '{pattern}': parameter '{part}' cannot follow a multiple parameter"
            );
        }

        if let Some(stripped) = part.strip_prefix(':') {
            if is_optional {
                seen_optional = true;

                let name = &stripped[..stripped.len() - 1];
                pat.push_str(&format!(r"(?:\s+(?P<{name}>\S+))?"));
            } else {
                if seen_optional {
                    panic!(
                        "Invalid pattern '{pattern}': mandatory parameter '{part}' cannot follow an optional parameter"
                    );
                }

                let name = stripped;
                pat.push_str(&format!(r"\s+(?P<{name}>\S+)"));
            }
        } else if let Some(stripped) = part.strip_prefix('*') {
            seen_multiple = true;

            if is_optional {
                seen_optional = true;

                let name = &stripped[..stripped.len() - 1];
                pat.push_str(&format!(r"(?:\s+(?P<{name}>.*))?"));
            } else {
                if seen_optional {
                    panic!(
                        "Invalid pattern '{pattern}': mandatory parameter '{part}' cannot follow an optional parameter"
                    );
                }

                let name = stripped;
                pat.push_str(&format!(r"\s+(?P<{name}>.*)"));
            }
        } else {
            if seen_optional {
                panic!(
                    "Invalid pattern '{pattern}': literal word '{part}' cannot follow an optional parameter"
                );
            }

            pat.push_str(&format!(r"\s+{}", regex::escape(part)));
        }
    }

    let re = regex::Regex::new(&pat).unwrap();

    fn extract_params(text: &str, re: &regex::Regex) -> Flow {
        if let Some(captures) = re.captures(text) {
            let mut params = HashMap::new();
            for name in re.capture_names().flatten() {
                if let Some(m) = captures.name(name) {
                    params.insert(name.to_string(), m.as_str().to_string());
                }
            }

            super::proceed_with(CommandParams(params))
        } else if re.is_match(text) {
            super::proceed()
        } else {
            super::stop()
        }
    }

    move |_, update| match update {
        Update::NewMessage(message) => {
            let text = message.text();
            extract_params(text, &re)
        }
        Update::CallbackQuery(query) => {
            let data = String::from_utf8_lossy(query.data()).to_string();
            extract_params(&data, &re)
        }
        Update::InlineQuery(query) => {
            let text = query.text();
            extract_params(text, &re)
        }
        _ => super::stop(),
    }
}

/// Pass if the message has a URL.
///
/// It extracts URLs directly from message's format entities, [`tl::enums::MessageEntity::Url`].
///
/// Note: if the `url` feature is enabled, it will also extracts URLs that weren't converted to format
/// entities, which can happen when using a broken client.
///
/// # Injects:
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
/// # Injects:
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
