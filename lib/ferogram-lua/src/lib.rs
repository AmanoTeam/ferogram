// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

//! Ferogram is a small framework for building Telegram bots using the [`grammers`] library.
//!
//! Lua extension.

#[allow(unused_imports)]
use ferogram::lua::*;
use mlua::{lua_module, prelude::*};

/// Ferogram Lua module.
#[lua_module]
fn ferogram_lua(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table()?;

    Ok(exports)
}
