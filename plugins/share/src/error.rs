// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Serialize, ser::Serializer};

/// Alias for the result type returned by this crate's functions.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type returned by this crate's APIs. Serialized as its [`Display`](std::fmt::Display)
/// string when it crosses the IPC boundary.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Failed to run a command on the mobile plugin implementation (Kotlin on Android, Swift on
    /// iOS). This includes the native side rejecting the call, e.g. because a file cannot be
    /// shared.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// Forwarded from a [`tauri::Error`], e.g. when resolving a scoped path fails.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// Forwarded from an [`std::io::Error`] raised while resolving a file to share.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The [`ShareData`](crate::ShareData) has no text, URL or files.
    #[error("nothing to share: set at least one of text, url and files")]
    Empty,
    /// The URL to share could not be parsed.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    /// The file is not allowed by the share scope.
    #[error("Not allowed to share file {0}")]
    ForbiddenPath(PathBuf),
    /// The platform has no system share sheet (Linux).
    #[error("sharing is not supported on the current platform")]
    UnsupportedPlatform,
    /// The share sheet is shown from the main thread, which did not report back.
    #[error("failed to show the share sheet from the main thread")]
    MainThread,
    /// Forwarded from a Windows API call, see [`windows::core::Error`].
    #[cfg(windows)]
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
