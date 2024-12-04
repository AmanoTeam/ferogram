// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Error module.

/// The error type for this library.
#[derive(Debug)]
pub struct Error {
    /// The kind of error.
    pub kind: ErrorKind,
    /// The error message.
    pub message: String,
}

impl Error {
    /// Creates a new timeout error.
    pub fn timeout(time: u64) -> Self {
        Self {
            kind: ErrorKind::Timeout,
            message: format!("Timeout reached after waiting for {} seconds", time),
        }
    }

    /// Creates a new unknown error.
    pub fn unknown() -> Self {
        Self {
            kind: ErrorKind::Unknown,
            message: "Unknown error".to_string(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

/// The kind of error.
#[derive(Debug, Default)]
pub enum ErrorKind {
    /// The time has run out
    Timeout,
    /// The error is unknown
    #[default]
    Unknown,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "Timeout"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}
