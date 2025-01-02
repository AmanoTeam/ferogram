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
//! Python extension.

use ferogram::py::*;
use pyo3::prelude::*;

/// Ferogram Python module.
#[pymodule]
fn ferogram_py(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<Chat>()?;
    module.add_class::<UserStatus>()?;

    module.add_class::<Context>()?;
    module.add_class::<Message>()?;

    Ok(())
}
