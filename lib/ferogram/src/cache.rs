// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cache module.

use std::{collections::HashMap, path::Path, sync::Arc};

use bincode::{
    Decode, Encode, config,
    de::Decoder,
    enc::Encoder,
    error::{DecodeError, EncodeError},
};
use grammers_client::types::PackedChat;
use tokio::sync::RwLock;

/// The cache.
#[derive(Clone, Debug, Default)]
pub struct Cache {
    /// The inner cache.
    inner: Arc<RwLock<InnerCache>>,
}

impl Cache {
    /// Load a previous cache instance from a file or create one if it doesn’t exist.
    pub fn load_file_or_create<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        // try to open the cache file.
        if let Ok(mut file) = std::fs::File::open(&path) {
            // get the standard config.
            let config = config::standard();

            // construct the inner cache.
            let inner: InnerCache = bincode::decode_from_std_read(&mut file, config)?;

            log::debug!("loaded {} chats from cache", inner.chats.len());

            Ok(Self {
                inner: Arc::new(RwLock::new(inner)),
            })
        } else {
            log::debug!("no cache was found, generating a new one");

            Ok(Self {
                inner: Arc::new(RwLock::new(InnerCache::default())),
            })
        }
    }

    /// Try to save the cache to a file.
    pub async fn save_to_file<P: AsRef<Path>>(&self, path: P) -> crate::Result<()> {
        // delete the cache file if it exists.
        if std::fs::exists(&path)? {
            std::fs::remove_file(&path)?;
        }

        // create the cache file.
        let mut file = std::fs::File::create(path)?;

        // get the standard config.
        let config = config::standard();

        // clone the inner.
        let inner = self.inner.write().await.clone();

        // write to the file.
        bincode::encode_into_std_write(inner, &mut file, config)?;

        Ok(())
    }

    /// Gets a saved chat by its ID.
    pub fn get_chat(&self, chat_id: i64) -> Option<PackedChat> {
        let inner = self.inner.try_read().expect("failed to get saved chats");

        inner.chats.get(&chat_id).cloned()
    }

    /// Saves a chat in the cache.
    pub(crate) async fn save_chat(&self, chat: PackedChat) -> crate::Result<()> {
        let mut inner = self.inner.write().await;

        if !inner.chat_exists(chat.id) {
            log::trace!("saved a new chat: {:?}", chat);

            inner.push_chat(chat);
        }

        Ok(())
    }
}

/// The inner cache.
#[derive(Clone, Debug, Default)]
struct InnerCache {
    /// The packed chat map.
    chats: HashMap<i64, PackedChat>,
}

impl InnerCache {
    /// Pushes a chat.
    pub fn push_chat(&mut self, chat: PackedChat) {
        self.chats.entry(chat.id).or_insert(chat);
    }

    /// Checks if a chat exists.
    pub fn chat_exists(&self, chat_id: i64) -> bool {
        self.chats.contains_key(&chat_id)
    }
}

impl Encode for InnerCache {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        // convert chats to bytes.
        let chats = self
            .chats
            .clone()
            .into_iter()
            .map(|(id, chat)| (id, chat.to_bytes()))
            .collect::<HashMap<_, _>>();

        Encode::encode(&chats, encoder)?;

        Ok(())
    }
}

impl<Context> Decode<Context> for InnerCache {
    fn decode<D: Decoder<Context = Context>>(decoder: &mut D) -> Result<Self, DecodeError> {
        // convert bytes to chats.
        let encoded_chats: HashMap<i64, [u8; 17]> = Decode::decode(decoder)?;
        let chats = encoded_chats
            .into_iter()
            .map(|(id, bytes)| {
                (
                    id,
                    PackedChat::from_bytes(&bytes).expect("failed to decode chat bytes"),
                )
            })
            .collect::<HashMap<_, _>>();

        Ok(Self { chats })
    }
}
