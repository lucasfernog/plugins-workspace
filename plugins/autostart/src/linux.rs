// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Linux pieces that complement `auto_launch`.

use std::path::PathBuf;

use crate::{escape::desktop_entry_is_disabled, home_dir, Error, Result};

/// `~/.config/autostart`, the directory `auto_launch` stores desktop entries in.
fn autostart_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".config").join("autostart"))
}

/// Returns whether the desktop entry for `app_name` exists and isn't turned off.
///
/// `auto_launch` only checks that the file exists, so it reported `true` after the user turned
/// the entry off in their desktop environment's settings (`Hidden=true` or
/// `X-GNOME-Autostart-enabled=false`).
pub(crate) fn is_enabled(app_name: &str) -> Result<bool> {
    let path = autostart_dir()?.join(format!("{app_name}.desktop"));
    match std::fs::read(&path) {
        Ok(content) => Ok(!desktop_entry_is_disabled(&String::from_utf8_lossy(
            &content,
        ))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        // the file exists but can't be read: keep reporting it as `auto_launch` does
        Err(_) => Ok(path.exists()),
    }
}

/// Creates `~/.config/autostart` and its parents.
///
/// `auto_launch` uses `fs::create_dir`, which fails when `~/.config` doesn't exist yet.
pub(crate) fn create_autostart_dir() -> Result<()> {
    std::fs::create_dir_all(autostart_dir()?).map_err(|e| Error::Anyhow(e.to_string()))
}
