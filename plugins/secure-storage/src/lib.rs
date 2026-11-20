// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Store secrets such as tokens and passwords in the operating system's secure storage.
//!
//! | Platform | Backend |
//! | -------- | ------- |
//! | Linux    | [Secret Service](https://specifications.freedesktop.org/secret-service/latest/) (GNOME Keyring, KWallet, ...) |
//! | macOS    | Keychain |
//! | Windows  | Credential Manager |
//! | Android  | AES-256-GCM with a key held in the Android Keystore |
//! | iOS      | Keychain |
//!
//! Items are scoped to the application: on desktop and iOS they are stored under the
//! app's identifier, on Android they live in the app's private storage.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::SecureStorage;
#[cfg(mobile)]
pub use mobile::SecureStorage;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the secure storage APIs.
pub trait SecureStorageExt<R: Runtime> {
    /// Returns a handle to the [`SecureStorage`] APIs.
    fn secure_storage(&self) -> &SecureStorage<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SecureStorageExt<R> for T {
    fn secure_storage(&self) -> &SecureStorage<R> {
        self.state::<SecureStorage<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("secure-storage")
        .invoke_handler(tauri::generate_handler![
            commands::get_item,
            commands::set_item,
            commands::remove_item
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let secure_storage = mobile::init(app, api)?;
            #[cfg(desktop)]
            let secure_storage = desktop::init(app, api)?;
            app.manage(secure_storage);
            Ok(())
        })
        .build()
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        Err(Error::EmptyKey)
    } else {
        Ok(())
    }
}
