// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Linux pieces that complement `auto_launch`.

use std::path::PathBuf;

use crate::{home_dir, Error, Result};

/// `~/.config/autostart`, the directory `auto_launch` stores desktop entries in.
fn autostart_dir() -> Result<PathBuf> {
    Ok(home_dir()?.join(".config").join("autostart"))
}

/// Creates `~/.config/autostart` and its parents.
///
/// `auto_launch` uses `fs::create_dir`, which fails when `~/.config` doesn't exist yet.
pub(crate) fn create_autostart_dir() -> Result<()> {
    std::fs::create_dir_all(autostart_dir()?).map_err(|e| Error::Anyhow(e.to_string()))
}
