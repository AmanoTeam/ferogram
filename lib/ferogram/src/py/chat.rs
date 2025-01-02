// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Chat module.

use grammers_client::{grammers_tl_types as tl, types};
use pyo3::prelude::*;

/// A chat.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Chat(types::Chat);

#[pymethods]
impl Chat {
    /// Chat id.
    #[getter]
    pub fn id(&self) -> i64 {
        self.0.id()
    }

    /// User phone.
    #[getter]
    pub fn phone(&self) -> Option<String> {
        match &self.0 {
            types::Chat::User(u) => u.phone().map(|p| p.to_string()),
            _ => None,
        }
    }

    /// User status.
    #[getter]
    pub fn status(&self) -> Option<UserStatus> {
        match &self.0 {
            types::Chat::User(u) => Some(u.status().into()),
            _ => None,
        }
    }

    /// User first name or group/channel title.
    #[getter]
    pub fn first_name(&self) -> Option<String> {
        match &self.0 {
            types::Chat::User(u) => u.first_name().map(ToString::to_string),
            types::Chat::Group(g) => g.title().map(ToString::to_string),
            types::Chat::Channel(c) => Some(c.title().to_string()),
        }
    }

    /// User last name.
    #[getter]
    pub fn last_name(&self) -> Option<String> {
        match &self.0 {
            types::Chat::User(u) => u.last_name().map(|n| n.to_string()),
            _ => None,
        }
    }

    /// Chat full name.
    #[getter]
    pub fn full_name(&self) -> Option<String> {
        match &self.0 {
            types::Chat::User(u) => Some(u.full_name().to_string()),
            types::Chat::Group(g) => g.title().map(ToString::to_string),
            types::Chat::Channel(c) => Some(c.title().to_string()),
        }
    }

    /// Chat username.
    #[getter]
    pub fn username(&self) -> Option<String> {
        self.0.username().map(|u| u.to_string())
    }

    /// Chat usernames.
    #[getter]
    pub fn usernames(&self) -> Vec<String> {
        self.0
            .usernames()
            .into_iter()
            .map(|u| u.to_string())
            .collect()
    }

    /// User language code.
    #[getter]
    pub fn lang_code(&self) -> Option<String> {
        match &self.0 {
            types::Chat::User(u) => u.lang_code().map(|c| c.to_string()),
            _ => None,
        }
    }

    /// The chat is a bot?
    #[getter]
    pub fn is_bot(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.is_bot())
    }

    /// The chat is a user?
    #[getter]
    pub fn is_user(&self) -> bool {
        matches!(&self.0, types::Chat::User(_))
    }

    /// The user has been flagged for scam
    #[getter]
    pub fn is_scam(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.scam())
    }

    /// The chat is the client itself?
    #[getter]
    pub fn is_self(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.is_self())
    }

    /// The chat is a group?
    #[getter]
    pub fn is_group(&self) -> bool {
        matches!(&self.0, types::Chat::Group(_))
    }

    /// The chat is a channel?
    #[getter]
    pub fn is_channel(&self) -> bool {
        matches!(&self.0, types::Chat::Channel(_))
    }

    /// This user is in the client's contact list?
    #[getter]
    pub fn is_contact(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.contact())
    }

    /// The user's account is deleted?
    #[getter]
    pub fn is_deleted(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.deleted())
    }

    /// The user is an official support team's member?
    #[getter]
    pub fn is_support(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.support())
    }

    /// The user has been verified?
    #[getter]
    pub fn is_verified(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.verified())
    }

    /// The user have restrictions?
    #[getter]
    pub fn is_restricted(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.restricted())
    }

    /// This user is in the client's mutual contact list and the user has the client in their
    /// contact list?
    #[getter]
    pub fn is_mutual_contact(&self) -> bool {
        matches!(&self.0, types::Chat::User(u) if u.mutual_contact())
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl From<types::Chat> for Chat {
    fn from(chat: types::Chat) -> Self {
        Self(chat)
    }
}

impl From<&types::Chat> for Chat {
    fn from(chat: &types::Chat) -> Self {
        Self(chat.clone())
    }
}

impl From<Chat> for types::Chat {
    fn from(chat: Chat) -> Self {
        chat.0
    }
}

impl From<&Chat> for types::Chat {
    fn from(chat: &Chat) -> Self {
        chat.0.clone()
    }
}

/// User status.
#[pyclass]
#[derive(Clone, Debug)]
pub enum UserStatus {
    Empty(),
    Online { expires: i32 },
    Offline { was_online: i32 },
    Recently { by_me: bool },
    LastWeek { by_me: bool },
    LastMonth { by_me: bool },
}

impl From<tl::enums::UserStatus> for UserStatus {
    fn from(status: tl::enums::UserStatus) -> Self {
        use tl::enums::UserStatus;

        match status {
            UserStatus::Empty => Self::Empty(),
            UserStatus::Online(o) => Self::Online { expires: o.expires },
            UserStatus::Offline(o) => Self::Offline {
                was_online: o.was_online,
            },
            UserStatus::Recently(r) => Self::Recently { by_me: r.by_me },
            UserStatus::LastWeek(l) => Self::LastWeek { by_me: l.by_me },
            UserStatus::LastMonth(l) => Self::LastMonth { by_me: l.by_me },
        }
    }
}

impl From<&tl::enums::UserStatus> for UserStatus {
    fn from(status: &tl::enums::UserStatus) -> Self {
        status.into()
    }
}
