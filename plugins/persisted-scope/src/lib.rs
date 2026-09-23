// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Save filesystem and asset scopes and restore them when the app is reopened.
//!
//! ## Cargo features
//!
//! - **protocol-asset**: Enables `tauri`'s `protocol-asset` feature and adds support for persisting `asset://` protocol scopes.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};
use tauri_plugin_fs::FsExt;

use std::{
    ffi::OsString,
    fs::{create_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

// Using 2 separate files so that we don't have to think about write conflicts and not break backwards compat.
const SCOPE_STATE_FILENAME: &str = ".persisted-scope";
const ASSET_SCOPE_STATE_FILENAME: &str = ".persisted-scope-asset";

/// All files the plugin writes under the app data directory (state files and their temporary files).
fn state_files(app_dir: &Path) -> Vec<PathBuf> {
    [SCOPE_STATE_FILENAME, ASSET_SCOPE_STATE_FILENAME]
        .into_iter()
        .flat_map(|name| {
            let path = app_dir.join(name);
            [temp_state_path(&path), path]
        })
        .collect()
}

/// Forbids access to every state file through `scope`, so the webview can't read or tamper
/// with the persisted grants of either scope (for instance through the fs plugin when the
/// app allows writing to the app data directory).
fn forbid_state_files(scope: &tauri::fs::Scope, app_dir: &Path) {
    for path in state_files(app_dir) {
        if let Err(e) = scope.forbid_file(&path) {
            log::warn!("failed to forbid access to {}: {e}", path.display());
        }
    }
}

// Most of these patterns are just added to try to fix broken files in the wild.
// After a while we can hopefully reduce it to something like [r"[?]", r"[*]", r"\\?\\\?\"]
const PATTERNS: &[&str] = &[
    r"[[]",
    r"[]]",
    r"[?]",
    r"[*]",
    r"\?\?",
    r"\\?\\?\",
    r"\\?\\\?\",
];
const REPLACE_WITH: &[&str] = &[r"[", r"]", r"?", r"*", r"\?", r"\\?\", r"\\?\"];

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Bincode(#[from] Box<bincode::ErrorKind>),
}

#[derive(Debug, Default, Deserialize, Serialize, Eq, PartialEq, Hash)]
enum TargetType {
    #[default]
    File,
    Directory,
    RecursiveDirectory,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Scope {
    allowed_paths: Vec<String>,
    forbidden_patterns: Vec<String>,
}

fn fix_pattern(ac: &AhoCorasick, s: &str) -> String {
    let s = ac.replace_all(s, REPLACE_WITH);

    if ac.find(&s).is_some() {
        return fix_pattern(ac, &s);
    }

    s
}

const RESURSIVE_DIRECTORY_SUFFIX: &str = "**";
const DIRECTORY_SUFFIX: &str = "*";

fn detect_scope_type(scope_state_path: &str) -> TargetType {
    if scope_state_path.ends_with(RESURSIVE_DIRECTORY_SUFFIX) {
        TargetType::RecursiveDirectory
    } else if scope_state_path.ends_with(DIRECTORY_SUFFIX) {
        TargetType::Directory
    } else {
        TargetType::File
    }
}

fn fix_directory(path_str: &str) -> &Path {
    let mut path = Path::new(path_str);

    if path.ends_with(DIRECTORY_SUFFIX) || path.ends_with(RESURSIVE_DIRECTORY_SUFFIX) {
        path = match path.parent() {
            Some(value) => value,
            None => return path,
        };
    }

    path
}

fn allow_path(scope: &tauri::fs::Scope, path: &str) {
    let target_type = detect_scope_type(path);

    match target_type {
        TargetType::File => {
            let _ = scope.allow_file(Path::new(path));
        }
        TargetType::Directory => {
            // We remove the '*' at the end of it, else it will be escaped by the pattern.
            let _ = scope.allow_directory(fix_directory(path), false);
        }
        TargetType::RecursiveDirectory => {
            // We remove the '**' at the end of it, else it will be escaped by the pattern.
            let _ = scope.allow_directory(fix_directory(path), true);
        }
    }
}

fn forbid_path(scope: &tauri::fs::Scope, path: &str) {
    let target_type = detect_scope_type(path);

    match target_type {
        TargetType::File => {
            let _ = scope.forbid_file(Path::new(path));
        }
        TargetType::Directory => {
            let _ = scope.forbid_directory(fix_directory(path), false);
        }
        TargetType::RecursiveDirectory => {
            let _ = scope.forbid_directory(fix_directory(path), true);
        }
    }
}

/// Serializes writes of the state files, so concurrent scope changes coming from different
/// threads can't interleave their writes or persist an older snapshot over a newer one.
static SAVE_LOCK: Mutex<()> = Mutex::new(());

/// Path of the temporary file used to atomically replace `state_path`.
fn temp_state_path(state_path: &Path) -> PathBuf {
    let mut file_name = state_path
        .file_name()
        .map(OsString::from)
        .unwrap_or_default();
    file_name.push(".tmp");
    state_path.with_file_name(file_name)
}

fn read_state(state_path: &Path) -> Result<Scope, Error> {
    let bytes = std::fs::read(state_path)?;
    Ok(bincode::deserialize(&bytes)?)
}

/// Writes the state to a temporary file first and then renames it over `state_path`,
/// so a crash or power loss mid-write can't leave a truncated state file behind.
fn write_state(state: &Scope, app_dir: &Path, state_path: &Path) -> Result<(), Error> {
    let bytes = bincode::serialize(state)?;
    create_dir_all(app_dir)?;

    let temp_path = temp_state_path(state_path);
    let result = File::create(&temp_path)
        .and_then(|mut f| {
            f.write_all(&bytes)?;
            f.sync_all()
        })
        .and_then(|_| std::fs::rename(&temp_path, state_path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp_path);
    }
    result.map_err(Into::into)
}

/// Restores the scope persisted at `state_path` and saves the repaired patterns back once.
///
/// If the file can't be read or decoded it is left untouched (instead of being overwritten
/// with an empty state), so the saved grants aren't silently discarded.
fn restore_scope(scope: &tauri::fs::Scope, ac: &AhoCorasick, app_dir: &Path, state_path: &Path) {
    let state = match read_state(state_path) {
        Ok(state) => state,
        Err(e) => {
            log::warn!(
                "failed to read the persisted scope at {}, it will not be restored: {e}",
                state_path.display()
            );
            return;
        }
    };

    for allowed in &state.allowed_paths {
        let allowed = fix_pattern(ac, allowed);
        allow_path(scope, &allowed);
    }
    for forbidden in &state.forbidden_patterns {
        let forbidden = fix_pattern(ac, forbidden);
        forbid_path(scope, &forbidden);
    }

    // Manually save the fixed scopes to disk once.
    // This is needed to fix broken .persisted-scope files in case the app doesn't update the scope itself.
    save_scopes(scope, app_dir, state_path);
}

fn save_scopes(scope: &tauri::fs::Scope, app_dir: &Path, scope_state_path: &Path) {
    // Hold the lock while taking the snapshot too, so the last write always has the newest state.
    let _guard = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let scope = Scope {
        allowed_paths: scope
            .allowed_patterns()
            .into_iter()
            .map(|p| p.to_string())
            .collect(),
        forbidden_patterns: scope
            .forbidden_patterns()
            .into_iter()
            .map(|p| p.to_string())
            .collect(),
    };

    if let Err(e) = write_state(&scope, app_dir, scope_state_path) {
        log::error!(
            "failed to persist the scope to {}: {e}",
            scope_state_path.display()
        );
    }
}

/// Initializes the plugin.
///
/// On setup, this restores the filesystem scope, and the `asset://` protocol scope when the
/// `protocol-asset` feature is enabled, from the state persisted during a previous run, then
/// listens for further scope changes to persist them again. Scopes are stored under the app's
/// data directory; nothing is restored or persisted if that directory cannot be resolved. The
/// `fs` plugin must be registered before this plugin, otherwise the filesystem scope is not
/// restored or persisted (a warning is printed in debug builds).
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("persisted-scope")
        .setup(|app, _api| {
            let fs_scope = app.try_fs_scope();
            #[cfg(feature = "protocol-asset")]
            let asset_protocol_scope = app.asset_protocol_scope();
            let app = app.clone();
            let app_dir = app.path().app_data_dir();

            if let Ok(app_dir) = app_dir {
                let fs_scope_state_path = app_dir.join(SCOPE_STATE_FILENAME);
                #[cfg(feature = "protocol-asset")]
                let asset_scope_state_path = app_dir.join(ASSET_SCOPE_STATE_FILENAME);

                if let Some(fs_scope) = &fs_scope {
                    forbid_state_files(fs_scope, &app_dir);
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("Please make sure to register the `fs` plugin before the `persisted-scope` plugin!");
                }
                #[cfg(feature = "protocol-asset")]
                forbid_state_files(&asset_protocol_scope, &app_dir);

                // We're trying to fix broken .persisted-scope files seamlessly, so we'll be running this on the values read on the saved file.
                // We will still save some semi-broken values because the scope events are quite spammy and we don't want to reduce runtime performance any further.
                let ac = AhoCorasick::new(PATTERNS).unwrap(/* This should be impossible to fail since we're using a small static input */);

                if let Some(fs_scope) = &fs_scope {
                    if fs_scope_state_path.exists() {
                        restore_scope(fs_scope, &ac, &app_dir, &fs_scope_state_path);
                    }
                }

                #[cfg(feature = "protocol-asset")]
                if asset_scope_state_path.exists() {
                    restore_scope(&asset_protocol_scope, &ac, &app_dir, &asset_scope_state_path);
                }

                #[cfg(feature = "protocol-asset")]
                let app_dir_ = app_dir.clone();

                if let Some(fs_scope) = &fs_scope {
                    fs_scope.listen(move |event| {
                        if let tauri::fs::Event::PathAllowed(_) = event {
                            save_scopes(&app.fs_scope(), &app_dir, &fs_scope_state_path);
                        }
                    });
                }

                #[cfg(feature = "protocol-asset")]
                {
                    let asset_protocol_scope_ = asset_protocol_scope.clone();
                    asset_protocol_scope.listen(move |event| {
                        if let tauri::scope::fs::Event::PathAllowed(_) = event {
                            save_scopes(&asset_protocol_scope_, &app_dir_, &asset_scope_state_path);
                        }
                    });
                }
            }
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tauri-plugin-persisted-scope-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn write_state_round_trips_and_leaves_no_temp_file() {
        let dir = test_dir("round-trip");
        let path = dir.join(SCOPE_STATE_FILENAME);
        let state = Scope {
            allowed_paths: vec!["/home/user/file.txt".into(), "/home/user/dir/**".into()],
            forbidden_patterns: vec!["/home/user/secret".into()],
        };

        write_state(&state, &dir, &path).unwrap();
        // Overwriting an existing file must work too (the rename replaces it).
        write_state(&state, &dir, &path).unwrap();

        let read = read_state(&path).unwrap();
        assert_eq!(read.allowed_paths, state.allowed_paths);
        assert_eq!(read.forbidden_patterns, state.forbidden_patterns);
        assert!(!temp_state_path(&path).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn truncated_state_is_an_error() {
        let dir = test_dir("truncated");
        let path = dir.join(SCOPE_STATE_FILENAME);
        let state = Scope {
            allowed_paths: vec!["/home/user/file.txt".into()],
            forbidden_patterns: vec![],
        };
        write_state(&state, &dir, &path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        std::fs::write(&path, &bytes[..bytes.len() / 2]).unwrap();
        assert!(read_state(&path).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_files_cover_both_scopes_and_temp_files() {
        let dir = Path::new("/data/app");
        let files = state_files(dir);
        for name in [
            ".persisted-scope",
            ".persisted-scope.tmp",
            ".persisted-scope-asset",
            ".persisted-scope-asset.tmp",
        ] {
            assert!(files.contains(&dir.join(name)), "missing {name}");
        }
    }

    #[test]
    fn temp_state_path_appends_suffix() {
        let path = Path::new("/data/app/.persisted-scope");
        assert_eq!(
            temp_state_path(path),
            Path::new("/data/app/.persisted-scope.tmp")
        );
    }
}
