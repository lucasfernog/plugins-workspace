// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// `platform`, `version`, `os_type`, `family`, `arch` and `exe_extension` are not
// commands (their values come from the init script); their deprecated no-op
// permissions live in `permissions/deprecated.toml`.
const COMMANDS: &[&str] = &["locale", "hostname"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .build();
}
