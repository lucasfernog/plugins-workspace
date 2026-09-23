// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use schemars::JsonSchema;

#[path = "src/scope_entry.rs"]
mod scope_entry;

/// A command argument allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Hash, schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellScopeEntryAllowedArg {
    /// A non-configurable argument that is passed to the command in the order it was specified.
    Fixed(String),

    /// A variable that is set while calling the command from the webview API.
    ///
    Var {
        /// [regex] validator to require passed values to conform to an expected input.
        ///
        /// This will require the argument value passed to this variable to match the `validator` regex
        /// before it will be executed.
        ///
        /// The regex string is by default surrounded by `^...$` to match the full string.
        /// For example the `https?://\w+` regex would be registered as `^https?://\w+$`.
        ///
        /// [regex]: <https://docs.rs/regex/latest/regex/#syntax>
        validator: String,

        /// Marks the validator as a raw regex, meaning the plugin should not make any modification at runtime.
        ///
        /// This means the regex will not match on the entire string by default, which might
        /// be exploited if your regex allow unexpected input to be considered valid.
        /// When using this option, make sure your regex is correct.
        #[serde(default)]
        raw: bool,
    },
}

/// A set of command arguments allowed to be executed by the webview API.
///
/// A value of `true` will allow any arguments to be passed to the command. `false` will disable all
/// arguments. A list of [`ShellScopeEntryAllowedArg`] will set those arguments as the only valid arguments to
/// be passed to the attached command configuration.
#[derive(Debug, PartialEq, Eq, Clone, Hash, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellScopeEntryAllowedArgs {
    /// Use a simple boolean to allow all or disable all arguments to this command configuration.
    Flag(bool),

    /// A specific set of [`ShellScopeEntryAllowedArg`] that are valid to call for the command configuration.
    List(Vec<ShellScopeEntryAllowedArg>),
}

impl Default for ShellScopeEntryAllowedArgs {
    fn default() -> Self {
        Self::Flag(false)
    }
}

/// The environment variables the webview API may set for a command.
///
/// A value of `true` (the default) allows the webview to set any environment variable. `false`
/// allows none, and a list of variable names only allows those. Clearing the environment with
/// `env: null` is always allowed.
#[derive(Debug, PartialEq, Eq, Clone, Hash, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellScopeEntryAllowedEnv {
    /// Allow (`true`) or disallow (`false`) setting any environment variable.
    Flag(bool),

    /// The names of the environment variables the webview may set.
    List(Vec<String>),
}

impl Default for ShellScopeEntryAllowedEnv {
    fn default() -> Self {
        Self::Flag(true)
    }
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

/// Shell scope entry.
#[derive(JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[allow(unused)]
pub(crate) enum ShellScopeEntry {
    Command {
        /// The name for this allowed shell command configuration.
        ///
        /// This name will be used inside of the webview API to call this command along with
        /// any specified arguments.
        name: String,
        /// The command name.
        /// It can start with a variable that resolves to a system base directory.
        /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
        /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
        /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
        /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
        // use default just so the schema doesn't flag it as required
        #[serde(rename = "cmd")]
        command: PathBuf,
        /// The allowed arguments for the command execution.
        #[serde(default)]
        args: ShellScopeEntryAllowedArgs,
        /// The environment variables the webview may set when calling this command
        /// (the `env` option of the webview API).
        ///
        /// `true` (the default) allows any variable, `false` allows none and a list only allows
        /// the variables it names. Variables such as `PATH`, `LD_PRELOAD` or
        /// `DYLD_INSERT_LIBRARIES` change which program runs or what code it loads,
        /// so restrict this for commands that untrusted content can call.
        #[serde(default)]
        env: ShellScopeEntryAllowedEnv,
        /// Whether the webview may set the working directory of this command
        /// (the `cwd` option of the webview API). Defaults to `true`.
        #[serde(default = "default_true")]
        cwd: bool,
    },
    Sidecar {
        /// The name for this allowed shell command configuration.
        ///
        /// This name will be used inside of the webview API to call this command along with
        /// any specified arguments.
        name: String,
        /// The allowed arguments for the command execution.
        #[serde(default)]
        args: ShellScopeEntryAllowedArgs,
        /// If this command is a sidecar command.
        sidecar: bool,
        /// The environment variables the webview may set when calling this command
        /// (the `env` option of the webview API).
        ///
        /// `true` (the default) allows any variable, `false` allows none and a list only allows
        /// the variables it names. Variables such as `PATH`, `LD_PRELOAD` or
        /// `DYLD_INSERT_LIBRARIES` change which program runs or what code it loads,
        /// so restrict this for commands that untrusted content can call.
        #[serde(default)]
        env: ShellScopeEntryAllowedEnv,
        /// Whether the webview may set the working directory of this command
        /// (the `cwd` option of the webview API). Defaults to `true`.
        #[serde(default = "default_true")]
        cwd: bool,
    },
}

// Ensure `ShellScopeEntry` and `scope_entry::EntryRaw`
// and `ShellScopeEntryAllowedArg` and `ShellAllowedArg`
// and `ShellScopeEntryAllowedArgs` and `ShellAllowedArgs`
// are kept in sync
#[allow(clippy::unnecessary_operation)]
fn _f() {
    match (ShellScopeEntry::Sidecar {
        name: String::new(),
        args: ShellScopeEntryAllowedArgs::Flag(false),
        sidecar: true,
        env: ShellScopeEntryAllowedEnv::Flag(true),
        cwd: true,
    }) {
        ShellScopeEntry::Command {
            name,
            command,
            args,
            env,
            cwd,
        } => scope_entry::EntryRaw {
            name,
            command: Some(command),
            args: match args {
                ShellScopeEntryAllowedArgs::Flag(flag) => scope_entry::ShellAllowedArgs::Flag(flag),
                ShellScopeEntryAllowedArgs::List(vec) => scope_entry::ShellAllowedArgs::List(
                    vec.into_iter()
                        .map(|s| match s {
                            ShellScopeEntryAllowedArg::Fixed(fixed) => {
                                scope_entry::ShellAllowedArg::Fixed(fixed)
                            }
                            ShellScopeEntryAllowedArg::Var { validator, raw } => {
                                scope_entry::ShellAllowedArg::Var { validator, raw }
                            }
                        })
                        .collect(),
                ),
            },
            sidecar: false,
            env: match env {
                ShellScopeEntryAllowedEnv::Flag(flag) => scope_entry::ShellAllowedEnv::Flag(flag),
                ShellScopeEntryAllowedEnv::List(list) => scope_entry::ShellAllowedEnv::List(list),
            },
            cwd,
        },
        ShellScopeEntry::Sidecar {
            name,
            args,
            sidecar,
            env,
            cwd,
        } => scope_entry::EntryRaw {
            name,
            command: None,
            args: match args {
                ShellScopeEntryAllowedArgs::Flag(flag) => scope_entry::ShellAllowedArgs::Flag(flag),
                ShellScopeEntryAllowedArgs::List(vec) => scope_entry::ShellAllowedArgs::List(
                    vec.into_iter()
                        .map(|s| match s {
                            ShellScopeEntryAllowedArg::Fixed(fixed) => {
                                scope_entry::ShellAllowedArg::Fixed(fixed)
                            }
                            ShellScopeEntryAllowedArg::Var { validator, raw } => {
                                scope_entry::ShellAllowedArg::Var { validator, raw }
                            }
                        })
                        .collect(),
                ),
            },
            sidecar,
            env: match env {
                ShellScopeEntryAllowedEnv::Flag(flag) => scope_entry::ShellAllowedEnv::Flag(flag),
                ShellScopeEntryAllowedEnv::List(list) => scope_entry::ShellAllowedEnv::List(list),
            },
            cwd,
        },
    };
}

const COMMANDS: &[&str] = &["execute", "spawn", "stdin_write", "kill", "open"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .global_scope_schema(schemars::schema_for!(ShellScopeEntry))
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mobile = target_os == "ios" || target_os == "android";
    alias("desktop", !mobile);
    alias("mobile", mobile);
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
    println!("cargo:rustc-check-cfg=cfg({alias})");
    if has_feature {
        println!("cargo:rustc-cfg={alias}");
    }
}
