// Copyright 2024 - Andriel Ferreira
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod plugin;

use std::{fs, path::Path};

use dlopen2::symbor::Library;
pub use plugin::Plugin;

use crate::Result;

/// Platform extension for shared libraries.
pub const PLATFORM_EXT: &str = if cfg!(target_os = "windows") {
    "dll"
} else if cfg!(target_os = "macos") {
    "dylib"
} else {
    "so"
};

/// Loads plugins from a directory.
pub(crate) fn load_plugins<P: AsRef<Path>>(path: P) -> Result<Vec<Plugin>> {
    let mut plugins = Vec::new();

    let files = fs::read_dir(&path)
        .expect("Failed to read plugins directory.")
        .map(|f| {
            f.expect("Failed to read file.")
                .file_name()
                .to_str()
                .expect("Failed to convert file name.")
                .split_once('.')
                .expect("Failed to split file name.")
                .0
                .to_owned()
        })
        .collect::<Vec<String>>();

    if files.is_empty() {
        log::info!("No plugins found in '{}'", path.as_ref().display());
        return Ok(plugins);
    }

    log::info!(
        "Plugins found in '{0}': {1}",
        path.as_ref().display(),
        files.join(", ")
    );

    for name in files.into_iter() {
        let path = format!("{0}/{1}.{2}", path.as_ref().display(), name, PLATFORM_EXT);

        let lib = Library::open(path).expect("Failed to load plugin.");
        let setup_fn = unsafe {
            lib.symbol::<extern "C" fn() -> Plugin>("setup")
                .expect("Failed to load setup function.")
        };

        let plugin = setup_fn();
        plugins.push(plugin);
    }

    log::info!("{} plugin(s) loaded", plugins.len());

    Ok(plugins)
}
