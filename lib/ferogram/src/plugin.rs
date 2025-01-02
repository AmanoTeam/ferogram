// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Plugin module.

use crate::{Handler, Router};

/// A plugin.
#[derive(Clone, Default)]
pub struct Plugin {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
    pub(crate) router: Router,
}

impl Plugin {
    /// Creates a new plugin builder.
    pub fn builder() -> PluginBuilder {
        PluginBuilder::default()
    }

    /// Returns the name of the plugin.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the version of the plugin.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the authors of the plugin.
    pub fn authors(&self) -> &Vec<String> {
        &self.authors
    }

    /// Returns the description of the plugin.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Adds a handler to the plugin.
    pub fn handler(mut self, handler: Handler) -> Self {
        self.router.handlers.push(handler);
        self
    }
}

/// A plugin builder.
#[derive(Default)]
pub struct PluginBuilder {
    name: String,
    version: String,
    authors: Vec<String>,
    description: String,
}

impl PluginBuilder {
    /// Sets the name of the plugin.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    /// Sets the version of the plugin.
    pub fn version(mut self, version: &str) -> Self {
        self.version = version.to_owned();
        self
    }

    /// Adds the author to the plugin.
    pub fn author(mut self, author: &str) -> Self {
        self.authors.push(author.to_owned());
        self
    }

    /// Sets the authors of the plugin.
    pub fn authors(mut self, authors: &[&str]) -> Self {
        self.authors = authors.into_iter().map(|a| a.to_string()).collect();
        self
    }

    /// Sets the description of the plugin
    pub fn description(mut self, description: &str) -> Self {
        self.description = description.to_owned();
        self
    }

    /// Builds the plugin.
    pub fn build(self) -> Plugin {
        Plugin {
            name: self.name,
            version: self.version,
            authors: self.authors,
            description: self.description,
            router: Router::default(),
        }
    }
}
