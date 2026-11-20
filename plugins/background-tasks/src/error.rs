// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, ser::Serializer};

/// Alias for the result type returned by the background tasks APIs.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur while scheduling or cancelling background tasks.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Background tasks are only available on Android and iOS.
    #[error("background tasks are not supported on this platform")]
    Unsupported,
    /// The task input could not be serialized to JSON.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Forwards an error returned by the underlying Android or iOS plugin invocation.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
