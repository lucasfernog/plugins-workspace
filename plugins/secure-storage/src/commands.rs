// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime, async_runtime::spawn_blocking, command};

use crate::{Result, SecureStorageExt};

// the credential stores block (D-Bus round trips, keychain prompts),
// so the calls are moved off the async runtime's worker threads

#[command]
pub(crate) async fn get_item<R: Runtime>(app: AppHandle<R>, key: String) -> Result<Option<String>> {
    spawn_blocking(move || app.secure_storage().get(&key)).await?
}

#[command]
pub(crate) async fn set_item<R: Runtime>(
    app: AppHandle<R>,
    key: String,
    value: String,
) -> Result<()> {
    spawn_blocking(move || app.secure_storage().set(&key, &value)).await?
}

#[command]
pub(crate) async fn remove_item<R: Runtime>(app: AppHandle<R>, key: String) -> Result<()> {
    spawn_blocking(move || app.secure_storage().remove(&key)).await?
}
