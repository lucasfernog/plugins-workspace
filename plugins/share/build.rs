// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

/// Share scope entry.
#[derive(schemars::JsonSchema)]
#[allow(unused)]
struct ShareScopeEntry {
    /// A file that the webview is allowed to share. Sharing text and URLs needs no scope.
    ///
    /// Wildcards can be used following the UNIX glob pattern.
    ///
    /// The pattern can start with a variable that resolves to a system base directory.
    /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
    /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
    /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
    /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
    path: PathBuf,
}

const COMMANDS: &[&str] = &["share"];

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .global_scope_schema(schemars::schema_for!(ShareScopeEntry))
        .android_path("android")
        .ios_path("ios")
        .try_build();

    // when building documentation for Android the plugin build result is always Err() (docs.rs mounts the sources read-only) and is irrelevant to the crate documentation build
    // `cfg(docsrs)` only reaches rustdoc, build scripts must check the `DOCS_RS` env var instead
    if !(std::env::var_os("DOCS_RS").is_some()
        && std::env::var("TARGET").unwrap().contains("android"))
    {
        result.unwrap();
    }
}
