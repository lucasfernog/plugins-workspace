// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS pieces that the plugin implements itself instead of delegating to `auto_launch`.

use std::process::Command;

use crate::{escape::applescript_string, Error, Result};

/// Runs `tell application "System Events" to <command>` through `osascript`.
fn run_system_events_script(command: &str) -> Result<()> {
    let script = format!("tell application \"System Events\" to {command}");
    let output = Command::new("osascript").arg("-e").arg(script).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(Error::Anyhow(format!(
            "Failed to execute apple script with status: {}: {}",
            output.status.code().unwrap_or(1),
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

/// Removes the login item named `name`, doing nothing if there is no such login item.
///
/// `auto_launch` runs `delete login item "<name>"`, which fails when the item doesn't exist.
pub(crate) fn delete_login_item(name: &str) -> Result<()> {
    let name = applescript_string(name);
    run_system_events_script(&format!(
        "if exists login item {name} then delete login item {name}"
    ))
}
