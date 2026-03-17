// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::io;

use grammers::{InvocationError, SignInError};
use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("failed to sign in: {0:?}")]
    SignInError(#[from] SignInError),
    #[error("failed to prompt: {0:?}")]
    PromptError(#[from] PromptError),
    #[error("failed to establish connection to database: {0:?}")]
    DatabaseError(#[from] libsql::Error),
    #[error("failed to establish connection to telegram: {0:?}")]
    ConnectionError(#[from] InvocationError),
    #[error("variable `{0}` were expected, but none was found")]
    ExpectedVariable(String),
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("an i/o error occurred: {0:?}")]
    IoError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum InjectorError {
    #[error("missing dependency: {0}")]
    MissingDependency(String),
}
