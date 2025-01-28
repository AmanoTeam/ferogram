// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod and;
mod command;
mod not;
mod or;

use std::sync::Arc;

pub(crate) use and::And;
pub(crate) use command::Command;
use grammers_client::{
    grammers_tl_types as tl,
    types::{Chat, Media},
    Client, Update,
};
pub(crate) use not::Not;
pub(crate) use or::Or;
use tokio::sync::Mutex;

use crate::{flow, Filter, Flow};

/// Default prefixes for commands.
pub const DEFAULT_PREFIXES: [&str; 2] = ["/", "!"];

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
    first.or(other)
}

/// Pass if `first` and `second` pass.
pub fn and<F: Filter, S: Filter>(first: F, second: S) -> impl Filter {
    first.and(second)
}

/// Pass if `filter` don't pass.
pub fn not<F: Filter>(filter: F) -> impl Filter {
    filter.not()
}

/// Pass if the message is from self.
pub async fn me(_: Client, update: Update) -> bool {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let sender = message.sender();

            if let Some(Chat::User(user)) = sender {
                user.is_self()
            } else {
                false
            }
        }
        Update::CallbackQuery(query) => {
            let sender = query.sender();

            if let Chat::User(user) = sender {
                user.is_self()
            } else {
                false
            }
        }
        Update::InlineQuery(query) => {
            let sender = query.sender();

            sender.is_self()
        }
        Update::InlineSend(inline_send) => {
            let sender = inline_send.sender();

            sender.is_self()
        }
        _ => false,
    }
}

/// Pass if the message contains the specified text.
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

/// Pass if the message text or query data matches the specified pattern.
pub fn regex(pat: &'static str) -> impl Filter {
    Arc::new(move |_client, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                regex::Regex::new(pat).unwrap().is_match(message.text())
            }
            Update::CallbackQuery(query) => regex::bytes::Regex::new(pat)
                .unwrap()
                .is_match(query.data()),
            Update::InlineQuery(query) => regex::Regex::new(pat).unwrap().is_match(query.text()),
            _ => false,
        }
    })
}

/// Pass if the message matches the specified command.
///
/// This filter is a custom [`regex`] filter, so it accepts regex syntax.
pub fn command(pat: &'static str) -> Command {
    Command {
        prefixes: DEFAULT_PREFIXES.into_iter().map(regex::escape).collect(),
        command: pat.to_owned(),
        description: String::new(),

        username: Arc::new(Mutex::new(None)),
    }
}

/// Pass if the message matches the specified command with custom prefixes.
///
/// This filter is a custom [`regex`] filter, so it accepts a bit of regex syntax.
pub fn command_with(pres: &'static [&'static str], pat: &'static str) -> Command {
    Command {
        prefixes: pres.iter().map(|pre| regex::escape(pre)).collect(),
        command: pat.to_owned(),
        description: String::new(),

        username: Arc::new(Mutex::new(None)),
    }
}

/// Pass if the message matches any of the specified commands.
pub fn commands(pats: &'static [&'static str]) -> Command {
    Command {
        prefixes: DEFAULT_PREFIXES.into_iter().map(regex::escape).collect(),
        command: pats.join("|"),
        description: String::new(),

        username: Arc::new(Mutex::new(None)),
    }
}

/// Pass if the message matches any of the specified commands with custom prefixes.
///
/// This filter is a custom [`regex`] filter, so it accepts a bit of regex syntax.
pub fn commands_with(pres: &'static [&'static str], pats: &'static [&'static str]) -> Command {
    Command {
        prefixes: pres.iter().map(|pre| regex::escape(pre)).collect(),
        command: pats.join("|"),
        description: String::new(),

        username: Arc::new(Mutex::new(None)),
    }
}

/// Pass if the message has a url.
///
/// Injects `Vec<String>`: urls.
pub async fn has_url(_: Client, update: Update) -> Flow {
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
                flow::break_now()
            } else {
                flow::continue_with(urls)
            }
        }
        _ => flow::break_now(),
    }
}

/// Pass if the messaage has a dice.
///
/// Injects `Dice`: message's dice.
pub async fn has_dice(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Dice(dice)) = message.media() {
                return flow::continue_with(dice);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has text or caption.
///
/// Injects `String`: message's text.
pub async fn has_text(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let text = message.text().to_string();
            if !text.is_empty() {
                return flow::continue_with(text);
            }

            flow::break_now()
        }
        Update::CallbackQuery(query) => {
            if let Ok(message) = query.load_message().await {
                let text = message.text().to_string();
                if !text.is_empty() {
                    return flow::continue_with(text);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has a poll.
///
/// Injects `Poll`: message's poll.
pub async fn has_poll(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Poll(poll)) = message.media() {
                return flow::continue_with(poll);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has an audio.
///
/// Injects `Document`: message's audio.
pub async fn has_audio(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Document(document)) = message.media() {
                if document.audio_title().is_some()
                    || document.performer().is_some()
                    || document
                        .mime_type()
                        .is_some_and(|mime| mime.starts_with("audio/"))
                {
                    return flow::continue_with(document);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has a photo.
///
/// Injects `Photo`: message's photo.
pub async fn has_photo(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(photo) = message.photo() {
                return flow::continue_with(photo);
            } else if let Some(Media::Photo(photo)) = message.media() {
                return flow::continue_with(photo);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has a video.
///
/// Injects `Document`: message's video.
pub async fn has_video(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Document(document)) = message.media() {
                if document
                    .mime_type()
                    .is_some_and(|mime| mime.starts_with("video/"))
                {
                    return flow::continue_with(document);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has a document.
///
/// Injects `Document`: message's document.
pub async fn has_document(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Document(document)) = message.media() {
                return flow::continue_with(document);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has a sticker.
///
/// Injects `Sticker`: message's sticker.
pub async fn has_sticker(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Sticker(sticker)) = message.media() {
                return flow::continue_with(sticker);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message has an animated sticker.
///
/// Injects `Document`: message's animated sticker.
pub async fn has_animated_sticker(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if let Some(Media::Document(document)) = message.media() {
                if document.is_animated() {
                    return flow::continue_with(document);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
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

/// Pass if the message is forwarded.
pub async fn forwarded(_: Client, update: Update) -> Flow {
    if let Update::NewMessage(message) = update {
        if message.forward_header().is_some() || message.forward_count().is_some() {
            return flow::continue_now();
        }
    }

    flow::break_now()
}

/// Pass if the message or callback query is sent by an administrator.
pub async fn administrator(client: Client, update: Update) -> Flow {
    let chat;
    let sender;

    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            chat = message.chat();
            sender = message.sender();
        }
        Update::CallbackQuery(query) => {
            chat = query.chat().clone();
            sender = Some(query.sender().clone());
        }
        _ => return flow::break_now(),
    }

    match chat {
        Chat::User(_) => return flow::continue_now(),
        _ => {
            if let Some(sender) = sender {
                if let Ok(tl::enums::channels::ChannelParticipant::Participant(
                    channel_participant,
                )) = client
                    .invoke(&tl::functions::channels::GetParticipant {
                        channel: chat
                            .pack()
                            .try_to_input_channel()
                            .expect("Invalid input channel"),
                        participant: sender.pack().to_input_peer(),
                    })
                    .await
                {
                    match channel_participant.participant {
                        tl::enums::ChannelParticipant::Admin(_)
                        | tl::enums::ChannelParticipant::Creator(_) => return flow::continue_now(),
                        _ => return flow::break_now(),
                    }
                }
            }
        }
    }

    flow::break_now()
}

/// Pass if the chat is private.
///
/// Injects `Chat`: private chat.
///         `User`: private chat.
pub async fn private(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let chat = message.chat();

            if let Chat::User(ref user) = chat {
                let mut flow = flow::continue_with(user.clone());
                flow.inject(chat);

                return flow;
            }

            flow::break_now()
        }
        Update::CallbackQuery(query) => {
            let chat = query.chat();

            if let Chat::User(user) = chat {
                let mut flow = flow::continue_with(user.clone());
                flow.inject(chat.clone());

                return flow;
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the chat is a group or a supergroup.
///
/// Injects `Chat`: group chat.
///         `Group`: group chat.
pub async fn group(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let chat = message.chat();

            if let Chat::Group(ref group) = chat {
                let mut flow = flow::continue_with(group.clone());
                flow.inject(chat);

                return flow;
            }

            flow::break_now()
        }
        Update::CallbackQuery(query) => {
            let chat = query.chat();

            if let Chat::Group(group) = chat {
                let mut flow = flow::continue_with(group.clone());
                flow.inject(chat.clone());

                return flow;
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the chat is a channel.
///
/// Injects `Chat`: channel.
///         `Channel`: channel.
pub async fn channel(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            let chat = message.chat();

            if let Chat::Channel(ref channel) = chat {
                let mut flow = flow::continue_with(channel.clone());
                flow.inject(chat);

                return flow;
            }

            flow::break_now()
        }
        Update::CallbackQuery(query) => {
            let chat = query.chat();

            if let Chat::Channel(channel) = chat {
                let mut flow = flow::continue_with(channel.clone());
                flow.inject(chat.clone());

                return flow;
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the chat id is the specified id.
///
/// Injects `Chat`: chat.
pub fn id(id: i64) -> impl Filter {
    Arc::new(move |_, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                let chat = message.chat();

                if chat.id() == id {
                    return flow::continue_with(chat);
                }

                flow::break_now()
            }
            Update::CallbackQuery(query) => {
                let chat = query.chat();

                if chat.id() == id {
                    return flow::continue_with(chat.clone());
                }

                flow::break_now()
            }
            _ => flow::break_now(),
        }
    })
}

/// Pass if the chat usernames contains the specified username.
///
/// The username cannot contain the "@" prefix.
///
/// Injects `Chat`: chat.
pub fn username(username: &'static str) -> impl Filter {
    Arc::new(move |_, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                let chat = message.chat();

                if chat.username() == Some(username) {
                    return flow::continue_with(chat);
                } else {
                    let usernames = chat.usernames();

                    if usernames.contains(&username) {
                        return flow::continue_with(chat);
                    }
                }

                flow::break_now()
            }
            Update::CallbackQuery(query) => {
                let chat = query.chat();

                if chat.username() == Some(username) {
                    return flow::continue_with(chat.clone());
                } else {
                    let usernames = chat.usernames();

                    if usernames.contains(&username) {
                        return flow::continue_with(chat.clone());
                    }
                }

                flow::break_now()
            }

            _ => flow::break_now(),
        }
    })
}

/// Pass if the chat usernames contains any of the specified usernames.
///
/// The usernames cannot contain the "@" prefix.
///
/// Injects `Chat`: chat.
pub fn usernames(usernames: &'static [&'static str]) -> impl Filter {
    Arc::new(move |_, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                let chat = message.chat();

                if let Some(chat_username) = chat.username() {
                    if usernames.contains(&chat_username) {
                        return flow::continue_with(chat);
                    }
                } else {
                    let chat_usernames = chat.usernames();

                    if chat_usernames
                        .iter()
                        .any(|username| usernames.contains(username))
                    {
                        return flow::continue_with(chat);
                    }
                }

                flow::break_now()
            }
            Update::CallbackQuery(query) => {
                let chat = query.chat();

                if let Some(chat_username) = chat.username() {
                    if usernames.contains(&chat_username) {
                        return flow::continue_with(chat.clone());
                    }
                } else {
                    let chat_usernames = chat.usernames();

                    if chat_usernames
                        .iter()
                        .any(|username| usernames.contains(username))
                    {
                        return flow::continue_with(chat.clone());
                    }
                }

                flow::break_now()
            }

            _ => flow::break_now(),
        }
    })
}

/// Pass if the message is a reply.
///
/// Injects `Message`: reply message.
pub async fn reply(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();
                return flow::continue_with(reply);
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has a dice.
///
/// Injects `Dice`: reply message's dice.
pub async fn reply_dice(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Dice(dice)) = reply.media() {
                    return flow::continue_with(dice);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and contains the specified text.
///
/// Injects `Message`: reply message.
pub fn reply_text(pat: &'static str) -> impl Filter {
    Arc::new(move |_, update| async move {
        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                if message.reply_to_message_id().is_some() {
                    let reply = message.get_reply().await.unwrap().unwrap();

                    if reply.text().contains(pat) {
                        return flow::continue_with(reply);
                    }
                }

                flow::break_now()
            }
            _ => flow::break_now(),
        }
    })
}

/// Pass if the message is a reply and has a poll.
///
/// Injects `Poll`: reply message's poll.
pub async fn reply_poll(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Poll(poll)) = reply.media() {
                    return flow::continue_with(poll);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has an audio.
///
/// Injects `Document`: reply message's audio.
pub async fn reply_audio(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Document(document)) = reply.media() {
                    if document.audio_title().is_some()
                        || document.performer().is_some()
                        || document
                            .mime_type()
                            .is_some_and(|mime| mime.starts_with("audio/"))
                    {
                        return flow::continue_with(document);
                    }
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has a photo.
///
/// Injects `Photo`: reply message's photo.
pub async fn reply_photo(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(photo) = reply.photo() {
                    return flow::continue_with(photo);
                } else if let Some(Media::Photo(photo)) = reply.media() {
                    return flow::continue_with(photo);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has a video.
///
/// Injects `Document`: reply message's video.
pub async fn reply_video(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Document(document)) = reply.media() {
                    if document
                        .mime_type()
                        .is_some_and(|mime| mime.starts_with("video/"))
                    {
                        return flow::continue_with(document);
                    }
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has a document.
///
/// Injects `Document`: reply message's document.
pub async fn reply_document(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Document(document)) = reply.media() {
                    return flow::continue_with(document);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has a sticker.
///
/// Injects `Sticker`: reply message's sticker.
pub async fn reply_sticker(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Sticker(sticker)) = reply.media() {
                    return flow::continue_with(sticker);
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}

/// Pass if the message is a reply and has an animated sticker.
///
/// Injects `Document`: reply message's animated sticker.
pub async fn reply_animated_sticker(_: Client, update: Update) -> Flow {
    match update {
        Update::NewMessage(message) | Update::MessageEdited(message) => {
            if message.reply_to_message_id().is_some() {
                let reply = message.get_reply().await.unwrap().unwrap();

                if let Some(Media::Document(document)) = reply.media() {
                    if document.is_animated() {
                        return flow::continue_with(document);
                    }
                }
            }

            flow::break_now()
        }
        _ => flow::break_now(),
    }
}
