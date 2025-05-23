// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Flow module.

use crate::{Result, di::Injector};

/// Represents the control flow of a filter.
#[derive(Debug, Default)]
pub struct Flow {
    /// The action.
    action: Action,
    /// The injector.
    pub(crate) injector: Injector,
}

impl Flow {
    /// Changes the current action to [`Action::Break`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let flow = unimplemented!();
    /// flow.to_break();
    /// # }
    /// ```
    pub fn to_break(&mut self) {
        self.action = Action::Break;
    }

    /// Changes the current action to [`Action::Continue`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let flow = unimplemented!();
    /// flow.to_continue();
    /// # }
    /// ```
    pub fn to_continue(&mut self) {
        self.action = Action::Continue;
    }

    /// Injects a value.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let flow = unimplemented!();
    /// flow.inject(String::from("Hello, world!"));
    /// # }
    /// ```
    pub fn inject<R: Clone + Send + Sync + 'static>(&mut self, value: R) {
        self.injector.insert(value);
    }

    /// Checks if the current action is [`Action::Break`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let flow = unimplemented!();
    /// let is_break = flow.is_break();
    /// # }
    /// ```
    pub fn is_break(&self) -> bool {
        matches!(self.action, Action::Break)
    }

    /// Checks if the current action is [`Action::Continue`].
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() {
    /// # let flow = unimplemented!();
    /// let is_continue = flow.is_continue();
    /// # }
    /// ```
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
        if value { continue_now() } else { break_now() }
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

/// Represents the next action that will be made in the handler.
#[derive(Debug, Default)]
pub enum Action {
    Break,
    #[default]
    Continue,
}

/// Creates a new flow with action [`Action::Break`].
pub fn break_now() -> Flow {
    Flow {
        action: Action::Break,
        ..Default::default()
    }
}

/// Creates a new flow with action [`Action::Continue`].
pub fn continue_now() -> Flow {
    Flow {
        action: Action::Continue,
        ..Default::default()
    }
}

/// Creates a new flow with action [`Action::Continue`] and inject a value.
pub fn continue_with<R: Clone + Send + Sync + 'static>(value: R) -> Flow {
    let mut flow = continue_now();
    flow.inject(value);

    flow
}
