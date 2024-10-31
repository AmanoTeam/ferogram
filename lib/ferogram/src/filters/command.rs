// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use async_trait::async_trait;
use grammers_client::{Client, Update};

use crate::{Filter, Flow};

pub struct Command {
    pub prefixes: Vec<String>,
    pub command: String,
}

#[async_trait]
impl Filter for Command {
    async fn check(&self, _: Client, update: Update) -> Flow {
        let pat = format!("^[{0}]{1}", self.prefixes.join("|"), self.command);

        match update {
            Update::NewMessage(message) | Update::MessageEdited(message) => {
                regex::Regex::new(&pat).unwrap().is_match(message.text())
            }
            _ => false,
        }
        .into()
    }
}
