// Copyright 2024-2025 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utils module.

use std::io::{BufRead, Write};

use grammers_client::button::Inline;

use crate::Result;

/// Ask the user in the terminal.
///
/// # Example
///
/// ```no_run
/// let token = ferogram::utils::prompt("Enter your token: ", false)?;
/// ```
pub fn prompt<T: ToString>(text: T, password: bool) -> Result<String> {
    let text = text.to_string();

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(text.as_bytes())?;
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

/// Convert bytes to string.
///
/// # Example
///
/// ```
/// use ferogram::utils::bytes_to_string;
///
/// let bytes = b"Hello, World!";
/// let string = bytes_to_string(bytes);
///
/// assert_eq!(string, "Hello, World!");
/// ```
pub fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// Splits a vector of `Inline` buttons into columns with a specified number of buttons per column.
///
/// # Arguments
///
/// * `buttons` - A vector of `Inline` buttons to be split into columns.
/// * `per_column` - The number of buttons each column should contain.
///
/// # Returns
///
/// A vector of vectors, where each inner vector represents a column of `Inline` buttons.
///
/// # Example
///
/// ```
/// let buttons = vec![button1, button2, button3, button4, button5];
/// let columns = split_btns_into_columns(buttons, 2);
/// assert_eq!(columns, vec![vec![button1, button2], vec![button3, button4], vec![button5]]);
/// ```
pub fn split_btns_into_columns(buttons: Vec<Inline>, per_column: usize) -> Vec<Vec<Inline>> {
    let mut columns = Vec::new();

    let mut column = Vec::with_capacity(per_column);
    for button in buttons.into_iter() {
        if column.len() == per_column {
            columns.push(column);
            column = Vec::with_capacity(per_column);
        }

        column.push(button);
    }

    if !column.is_empty() {
        columns.push(column);
    }

    columns
}

/// Splits a vector of `Inline` buttons into rows with a specified number of rows.
///
/// # Arguments
///
/// * `buttons` - A vector of `Inline` buttons to be split into rows.
/// * `row_count` - The number of rows to split the buttons into.
///
/// # Returns
///
/// A vector of vectors, where each inner vector represents a row of `Inline` buttons.
///
/// # Example
///
/// ```no_run
/// let buttons = vec![button1, button2, button3, button4, button5];
/// let rows = split_btns_into_rows(buttons, 2);
/// assert_eq!(rows, vec![vec![button1, button2, button3], vec![button4, button5]]);
/// ```
pub fn split_btns_into_rows(buttons: Vec<Inline>, row_count: usize) -> Vec<Vec<Inline>> {
    let per_column = buttons.len().abs_diff(row_count);
    split_btns_into_columns(buttons, per_column)
}
