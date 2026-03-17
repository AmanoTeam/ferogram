// Copyright 2024-2026 - Andriel Ferreira
//
// Licensed under the MIT license <LICENSE or https://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except according to those terms.

use std::io::{BufRead, Write};

use crate::error::PromptError;

/// Ask a question to the user in the terminal.
pub fn prompt(question: &str, hide_chars: bool) -> Result<String, PromptError> {
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(question.as_bytes())?;
    stdout.flush()?;
    drop(stdout);

    let mut line = String::new();
    if hide_chars {
        line = rpassword::read_password()?;
    } else {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        stdin.read_line(&mut line)?;
    }

    Ok(line)
}
