// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::sync::Arc;

use async_trait::async_trait;
use grammers_client::{Client, Update};
use tokio::sync::Mutex;

use crate::{Filter, Flow};

#[derive(Clone)]
pub struct Command {
    pub(crate) prefixes: Vec<String>,
    pub(crate) command: String,

    pub(crate) username: Arc<Mutex<Option<String>>>,
}

#[async_trait]
impl Filter for Command {
    async fn check(&mut self, client: Client, update: Update) -> Flow {
        let command = self.command.clone();
        let splitted = command.split_whitespace().collect::<Vec<_>>();

        let mut username = self.username.lock().await;
        if username.is_none() {
            let me = client.get_me().await.unwrap();

            *username = me.username().map(|u| u.to_string());
        }

        let mut pat = String::new();
        if username.is_some() {
            pat += &format!("{0}(@{1})?", splitted[0], username.as_deref().unwrap());
        }

        let pre_pat = format!("^({})(?i)", self.prefixes.join("|"));
        if splitted.len() > 1 {
            pat = format!(r"{0}({1} {2})($|\s)", pre_pat, pat, splitted[1..].join(" "));
        } else {
            pat = format!(r"{0}({1})($|\s)", pre_pat, pat);
        }

        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                regex::Regex::new(&pat).unwrap().is_match(message.text())
            }
            _ => false,
        }
        .into()
    }
}
