// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Windows pieces that complement `auto_launch`.

use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_SET_VALUE},
    RegKey,
};

/// The Task Manager "Startup apps" override that `auto_launch` writes next to the `Run` value.
const STARTUP_APPROVED_RUN_KEY: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\StartupApproved\\Run";

/// Removes the Task Manager override `auto_launch` sets for `app_name` when enabling, which its
/// `disable` leaves behind.
///
/// Best effort: the key or value may not exist, and the `Run` value is what actually starts the
/// app, so errors are ignored.
pub(crate) fn remove_startup_approved_value(app_name: &str) {
    if let Ok(key) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(STARTUP_APPROVED_RUN_KEY, KEY_SET_VALUE)
    {
        let _ = key.delete_value(app_name);
    }
}
