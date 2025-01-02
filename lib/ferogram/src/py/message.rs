// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Message module.

use grammers_client::types;
use pyo3::{
    prelude::*,
    types::{timezone_utc, PyDateTime},
};

use super::Chat;

/// A message.
#[pyclass]
#[derive(Clone, Debug)]
pub struct Message(types::Message);

#[pymethods]
impl Message {
    /// The ID of the message.
    #[getter]
    pub fn id(&self) -> i32 {
        self.0.id()
    }

    /// The chat of the message.
    #[getter]
    pub fn chat(&self) -> Chat {
        self.0.chat().into()
    }

    /// The text of the message.
    #[getter]
    pub fn text(&self) -> String {
        self.0.text().to_string()
    }

    /// The date of the message.
    #[getter]
    pub fn date<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDateTime>> {
        PyDateTime::from_timestamp(
            py,
            self.0.date().timestamp() as f64,
            Some(&timezone_utc(py)),
        )
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self.0)
    }

    pub fn __repr__(&self) -> String {
        format!("{:?}", self.0)
    }
}

impl From<types::Message> for Message {
    fn from(msg: types::Message) -> Self {
        Self(msg)
    }
}

impl From<&types::Message> for Message {
    fn from(msg: &types::Message) -> Self {
        Self(msg.clone())
    }
}

impl From<Message> for types::Message {
    fn from(msg: Message) -> Self {
        msg.0
    }
}

impl From<&Message> for types::Message {
    fn from(msg: &Message) -> Self {
        msg.0.clone()
    }
}
