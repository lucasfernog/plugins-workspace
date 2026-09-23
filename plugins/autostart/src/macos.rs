// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! macOS pieces that the plugin implements itself instead of delegating to `auto_launch`.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    escape::{applescript_string, launch_agent_plist},
    Error, Result,
};

/// The checks `auto_launch` runs on the registered path before enabling auto start.
fn check_app_path(app_path: &str) -> Result<()> {
    let path = Path::new(app_path);
    if !path.exists() {
        return Err(Error::Anyhow(format!("app path doesn't exist: {app_path}")));
    }
    if !path.is_absolute() {
        return Err(Error::Anyhow(format!(
            "app path is not absolute: {app_path}"
        )));
    }
    Ok(())
}

/// `~/Library/LaunchAgents`, the directory `auto_launch` stores Launch Agents in.
fn launch_agents_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join("Library").join("LaunchAgents"))
        .ok_or_else(|| Error::Anyhow("failed to resolve the home directory".into()))
}

/// Writes the Launch Agent that starts `app_path` with `args` at login.
///
/// This replaces `auto_launch`'s implementation, which doesn't escape the values it writes to
/// the property list, so a `&` or `<` in the name, path or arguments made launchd ignore it.
/// The file name and content are otherwise the same, so `auto_launch` still finds it in
/// `is_enabled` and `disable`.
pub(crate) fn write_launch_agent(app_name: &str, app_path: &str, args: &[String]) -> Result<()> {
    check_app_path(app_path)?;

    let dir = launch_agents_dir()?;
    fs::create_dir_all(&dir)
        .and_then(|()| {
            fs::write(
                dir.join(format!("{app_name}.plist")),
                launch_agent_plist(app_name, app_path, args),
            )
        })
        .map_err(|e| Error::Anyhow(e.to_string()))
}

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

/// Adds a login item named `name` that opens `app_path`, hidden if `args` contains
/// `--hidden` or `--minimized` (the only arguments a login item supports).
///
/// This replaces `auto_launch`'s implementation, which interpolates the name and path into
/// the script without escaping, so a `"` in the app path broke (or injected into) the script.
pub(crate) fn add_login_item(name: &str, app_path: &str, args: &[String]) -> Result<()> {
    check_app_path(app_path)?;
    let hidden = args
        .iter()
        .any(|arg| arg == "--hidden" || arg == "--minimized");
    run_system_events_script(&format!(
        "make login item at end with properties {{name:{}, path:{}, hidden:{hidden}}}",
        applescript_string(name),
        applescript_string(app_path),
    ))
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
