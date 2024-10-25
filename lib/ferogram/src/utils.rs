// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{BufRead, Write};

use crate::Result;

/// Ask the user in the terminal.
pub fn prompt(message: impl Into<String>, password: bool) -> Result<String> {
    let message = message.into();

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;
    drop(stdout);

    let mut line = String::new();
    if password {
        line = rpassword::read_password()?;
    } else {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        stdin.read_line(&mut line)?;
    }

    Ok(line)
}
