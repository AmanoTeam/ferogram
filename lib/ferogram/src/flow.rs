// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Flow module.

use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{di::Injector, Result};

/// Represents the control flow of a handler's filter and its endpoint.
#[derive(Debug, Default)]
pub struct Flow {
    action: Action,
    pub(crate) injector: Arc<Mutex<Injector>>,
}

impl Flow {
    /// Change the current action to [`Action::Break`].
    pub fn to_break(&mut self) {
        self.action = Action::Break;
    }

    /// Change the current action to [`Action::Continue`].
    pub fn to_continue(&mut self) {
        self.action = Action::Continue;
    }

    /// Inject a value.
    pub fn inject<R: Clone + Send + Sync + 'static>(&mut self, value: R) {
        self.injector.try_lock().unwrap().insert(value);
    }

    /// Check if the current action is [`Action::Break`].
    pub fn is_break(&self) -> bool {
        matches!(self.action, Action::Continue)
    }

    /// Check if the current action is [`Action::Continue`].
    pub fn is_continue(&self) -> bool {
        matches!(self.action, Action::Continue)
    }
}

impl From<()> for Flow {
    fn from(_: ()) -> Self {
        break_now()
    }
}

impl From<bool> for Flow {
    fn from(value: bool) -> Self {
        if value {
            continue_now()
        } else {
            break_now()
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<Option<T>> for Flow {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => continue_with(value),
            None => break_now(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> From<Result<T>> for Flow {
    fn from(value: Result<T>) -> Self {
        match value {
            Ok(value) => continue_with(value),
            Err(_) => break_now(),
        }
    }
}

/// Represents the next action will be made onto the handler.
#[derive(Debug, Default)]
pub enum Action {
    Break,
    #[default]
    Continue,
}

/// Create a new flow with action [`Action::Break`].
pub fn break_now() -> Flow {
    Flow {
        action: Action::Break,
        ..Default::default()
    }
}

/// Create a new flow with action [`Action::Continue`].
pub fn continue_now() -> Flow {
    Flow {
        action: Action::Continue,
        ..Default::default()
    }
}

/// Create a new flow with action [`Action::Continue`] and inject a value.
pub fn continue_with<R: Clone + Send + Sync + 'static>(value: R) -> Flow {
    let mut flow = continue_now();
    flow.inject(value);

    flow
}
