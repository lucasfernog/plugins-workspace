// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::validate_key;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.securestorage";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_secure_storage);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<SecureStorage<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "SecureStoragePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_secure_storage)?;
    Ok(SecureStorage(handle))
}

/// Access to the secure storage APIs.
pub struct SecureStorage<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> SecureStorage<R> {
    /// Returns the value stored under `key`, or `None` if there is none.
    ///
    /// On Android this also returns `None` when the stored value can no longer be
    /// decrypted, which happens when the app data was restored from a backup: the
    /// encryption key never leaves the device's Keystore.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn get(&self, key: &str) -> crate::Result<Option<String>> {
        validate_key(key)?;
        self.0
            .run_mobile_plugin::<GetItemResponse>("getItem", KeyPayload { key })
            .map(|response| response.value)
            .map_err(Into::into)
    }

    /// Stores `value` under `key`, replacing any existing value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn set(&self, key: &str, value: &str) -> crate::Result<()> {
        validate_key(key)?;
        self.0
            .run_mobile_plugin("setItem", SetItemPayload { key, value })
            .map_err(Into::into)
    }

    /// Removes the value stored under `key`. Does nothing if there is none.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn remove(&self, key: &str) -> crate::Result<()> {
        validate_key(key)?;
        self.0
            .run_mobile_plugin("removeItem", KeyPayload { key })
            .map_err(Into::into)
    }
}

#[derive(Serialize)]
struct KeyPayload<'a> {
    key: &'a str,
}

#[derive(Serialize)]
struct SetItemPayload<'a> {
    key: &'a str,
    value: &'a str,
}

#[derive(Deserialize)]
struct GetItemResponse {
    #[serde(default)]
    value: Option<String>,
}
