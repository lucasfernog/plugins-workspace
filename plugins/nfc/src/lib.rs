// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Read and write NFC tags on Android and iOS.
//!
//! This plugin is mobile only: the whole crate is gated behind `#[cfg(mobile)]`,
//! so it expands to nothing when compiling for Linux, macOS or Windows.
//!
//! Register the plugin with `init` and use the `NfcExt` trait to reach the
//! `Nfc` instance from any `tauri::Manager` implementation (the app handle, a window, ...).

#![cfg(mobile)]

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.nfc";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_nfc);

/// Access to the nfc APIs.
pub struct Nfc<R: Runtime>(PluginHandle<R>);

#[derive(Deserialize)]
struct IsAvailableResponse {
    available: bool,
}

#[derive(Serialize)]
struct WriteRequest {
    records: Vec<NfcRecord>,
    #[serde(flatten)]
    options: WriteOptions,
}

impl<R: Runtime> Nfc<R> {
    /// Checks whether NFC is supported by the device and currently usable by the app.
    ///
    /// On Android this is `false` when the device has no NFC adapter or when NFC is
    /// disabled in the device settings.
    /// On iOS this is `false` when the `NFCReaderUsageDescription` entry is missing from the
    /// `Info.plist` file or when NFC tag reading is not available on the device.
    pub fn is_available(&self) -> crate::Result<bool> {
        self.0
            .run_mobile_plugin::<IsAvailableResponse>("isAvailable", ())
            .map(|r| r.available)
            .map_err(Into::into)
    }

    /// Scans an NFC tag, blocking until a tag matching the given [`ScanRequest::kind`] filters
    /// is read or the scan fails.
    ///
    /// Set [`ScanRequest::keep_session_alive`] to `true` to keep the connection to the tag open
    /// after it has been scanned, so that a following [`Self::write`] call writes to that tag.
    ///
    /// # Errors
    ///
    /// Returns an error when NFC is not available (see [`Self::is_available`])
    /// or when the tag could not be read.
    pub fn scan(&self, payload: ScanRequest) -> crate::Result<ScanResponse> {
        self.0
            .run_mobile_plugin("scan", payload)
            .map(|v| ScanResponse { tag: v })
            .map_err(Into::into)
    }

    /// Writes the given NDEF records to an NFC tag, blocking until the write completes or fails.
    ///
    /// Because this API does not take a scan kind, on Android it can only write to the tag of an
    /// ongoing session, so it must be preceded by a [`Self::scan`] call with
    /// [`ScanRequest::keep_session_alive`] set to `true`.
    /// On iOS an NDEF reader session is started when there is no ongoing session, and the
    /// records are written to the first tag that is scanned.
    ///
    /// # Errors
    ///
    /// Returns an error when NFC is not available (see [`Self::is_available`]), when there is no
    /// connected tag on Android, when the tag is read-only, when it cannot hold the message or
    /// when it does not support the NDEF format.
    pub fn write(&self, records: Vec<NfcRecord>) -> crate::Result<()> {
        self.write_with_options(records, WriteOptions::default())
    }

    /// Writes the given NDEF records to an NFC tag, blocking until the write completes or fails.
    ///
    /// Unlike [`Self::write`], this can set [`WriteOptions::kind`], so on Android it can scan
    /// for the tag to write to by itself instead of requiring a kept-alive [`Self::scan`] session,
    /// and the messages displayed in the iOS UI.
    ///
    /// When a [`Self::scan`] session is still active (e.g. kept alive), the records are written to
    /// its tag and [`WriteOptions::kind`] is ignored.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_nfc::{NfcExt, NfcRecord, NFCTypeNameFormat, ScanKind, WriteOptions};
    ///
    /// fn write_url<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> tauri_plugin_nfc::Result<()> {
    ///     // well known URI record, `0x04` is the `https://` prefix code
    ///     let mut payload = vec![0x04];
    ///     payload.extend_from_slice(b"tauri.app");
    ///     let record = NfcRecord {
    ///         format: NFCTypeNameFormat::NfcWellKnown,
    ///         kind: vec![0x55],
    ///         id: vec![],
    ///         payload,
    ///     };
    ///     app.nfc().write_with_options(
    ///         vec![record],
    ///         WriteOptions::new()
    ///             .kind(ScanKind::Ndef { mime_type: None, uri: None, tech_list: None })
    ///             .message("Hold your device near the tag"),
    ///     )
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Same as [`Self::write`].
    pub fn write_with_options(
        &self,
        records: Vec<NfcRecord>,
        options: WriteOptions,
    ) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("write", WriteRequest { records, options })
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the NFC APIs.
pub trait NfcExt<R: Runtime> {
    /// Returns the [`Nfc`] instance managed by the plugin.
    fn nfc(&self) -> &Nfc<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NfcExt<R> for T {
    fn nfc(&self) -> &Nfc<R> {
        self.state::<Nfc<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nfc")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "NfcPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_nfc)?;
            app.manage(Nfc(handle));
            Ok(())
        })
        .build()
}
