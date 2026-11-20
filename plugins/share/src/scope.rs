// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime, ipc::ScopeObject, utils::acl::Value};

use crate::Error;

#[derive(Deserialize)]
struct EntryRaw {
    path: PathBuf,
}

/// A file the webview is allowed (or denied) to share.
#[derive(Debug)]
pub struct Entry {
    // `None` when the pattern uses a base directory that does not exist on this platform
    path: Option<PathBuf>,
}

impl ScopeObject for Entry {
    type Error = Error;

    fn deserialize<R: Runtime>(
        app_handle: &AppHandle<R>,
        raw: Value,
    ) -> std::result::Result<Self, Self::Error> {
        let raw: EntryRaw = serde_json::from_value(raw.into()).map_err(tauri::Error::from)?;
        let path = match app_handle.path().parse(raw.path) {
            Ok(path) => Some(path),
            #[cfg(not(target_os = "android"))]
            Err(tauri::Error::UnknownPath) => None,
            Err(err) => return Err(err.into()),
        };
        Ok(Self { path })
    }
}

pub(crate) fn is_path_allowed<'a, R: Runtime, M: Manager<R>>(
    manager: &M,
    allowed: impl Iterator<Item = &'a Arc<Entry>>,
    denied: impl Iterator<Item = &'a Arc<Entry>>,
    path: &Path,
) -> crate::Result<bool> {
    let scope = tauri::fs::Scope::new(
        manager,
        &tauri::utils::config::FsScope::Scope {
            allow: allowed.filter_map(|e| e.path.clone()).collect(),
            deny: denied.filter_map(|e| e.path.clone()).collect(),
            require_literal_leading_dot: None,
        },
    )?;
    Ok(scope.is_allowed(path))
}
