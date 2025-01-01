// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Context module.

use pyo3::prelude::*;

use super::{Chat, Message};

/// The context of an update.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Context(crate::Context);

#[pymethods]
impl Context {
    /// The chat of the update.
    #[getter]
    pub fn chat(&self) -> Option<Chat> {
        self.0.chat().map(|c| c.into())
    }

    /// The text of the update.
    #[getter]
    pub fn text(&self) -> Option<String> {
        self.0.text()
    }

    /// The sender of the update.
    #[getter]
    pub fn sender(&self) -> Option<Chat> {
        self.0.sender().map(|s| s.into())
    }

    /// The query of the update.
    #[getter]
    pub fn query(&self) -> Option<String> {
        self.0.query()
    }

    /// Gets the message of the update.
    pub async fn message(&self) -> Option<Message> {
        self.0.message().await.map(|m| m.into())
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl From<crate::Context> for Context {
    fn from(ctx: crate::Context) -> Self {
        Self(ctx)
    }
}

impl From<&crate::Context> for Context {
    fn from(ctx: &crate::Context) -> Self {
        Self(ctx.clone())
    }
}

impl From<Context> for crate::Context {
    fn from(ctx: Context) -> Self {
        ctx.0
    }
}

impl From<&Context> for crate::Context {
    fn from(ctx: &Context) -> Self {
        ctx.0.clone()
    }
}
