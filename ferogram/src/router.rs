// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

//! Router definitions.

use std::{collections::HashMap, str::FromStr};

/// Stores extracted parameters like `:id` from the [`crate::filter::command`] filter.
///
/// Note: it won't be injected if the command pattern don't expect parameters.
#[derive(Clone, Debug, Default)]
pub struct CommandParams(pub(crate) HashMap<String, String>);

impl CommandParams {
    /// Get mandatory parameters like `:id` unparsed.
    pub fn get(&self, key: &str) -> Result<String, String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| format!("Missing parameter: {key}"))
    }

    /// Get mandatory parameters like `:id`.
    pub fn get_parsed<T: FromStr>(&self, key: &str) -> Result<T, String> {
        self.0
            .get(key)
            .ok_or_else(|| format!("Missing parameter: {key}"))?
            .parse::<T>()
            .map_err(|_| format!("Failed to parse parameter: {key}"))
    }

    /// Get optional parameters like `:id?` unparsed.
    pub fn get_optional(&self, key: &str) -> Result<Option<String>, String> {
        match self.0.get(key).cloned() {
            Some(value) => Ok(Some(value)),
            None => Ok(None),
        }
    }

    /// Get optional parameters like `:id?`.
    pub fn get_optional_parsed<T: FromStr>(&self, key: &str) -> Result<Option<T>, String> {
        match self.0.get(key) {
            Some(value) => {
                let parsed = value
                    .parse::<T>()
                    .map_err(|_| format!("Failed to parse optional parameter: {key}"))?;

                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }
}
