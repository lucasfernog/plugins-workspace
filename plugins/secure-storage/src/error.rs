// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, ser::Serializer};

/// Alias for a [`std::result::Result`] with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the secure storage APIs.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The key is an empty string.
    #[error("the key must not be empty")]
    EmptyKey,
    /// The operating system's credential store rejected the operation or could not be
    /// reached, e.g. because no Secret Service provider is running on Linux, the keychain
    /// is locked, or the value exceeds the platform's size limit. Only produced on desktop.
    #[cfg(desktop)]
    #[error(transparent)]
    Keyring(#[from] keyring_core::Error),
    /// The call to the Android or iOS plugin implementation failed, either because the
    /// arguments could not be serialized, the response could not be deserialized, or the
    /// native side rejected the call. Only produced on Android and iOS.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// The blocking task running the operation failed to complete.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
