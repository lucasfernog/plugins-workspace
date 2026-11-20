// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Share text, links and files through the system share sheet.
//!
//! - Supported platforms: Android, iOS, macOS and Windows. Linux has no system share sheet, so
//!   sharing fails there with [`Error::UnsupportedPlatform`].
//!
//! ## Security
//!
//! The `share` command lets the webview share text and URLs freely, but only the files allowed
//! by the plugin scope (`{ "path": "$APPDATA/**" }` entries on the `share:allow-share`
//! permission). [`Share::share`] is called by trusted Rust code and does not check the scope.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    Manager, Runtime, WebviewWindow,
    plugin::{Builder, TauriPlugin},
};

mod commands;
#[cfg(desktop)]
mod desktop;
mod error;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(mobile)]
mod mobile;
mod models;
mod scope;
#[cfg(windows)]
mod windows;

pub use error::{Error, Result};
pub use models::{Rect, ShareData, ShareOptions};

#[cfg(desktop)]
use desktop::Share as Inner;
#[cfg(mobile)]
use mobile::Share as Inner;

/// Access to the share APIs.
pub struct Share<R: Runtime>(Inner<R>);

impl<R: Runtime> Share<R> {
    /// Opens the system share sheet for the given content.
    ///
    /// Returns once the share sheet is shown. Whether the user completed the share or dismissed
    /// the sheet is not reported, because most platforms do not tell.
    ///
    /// `window` is the window the share sheet belongs to. It is unused on mobile.
    ///
    /// # Errors
    ///
    /// - [`Error::Empty`] if `data` has no text, URL or files.
    /// - [`Error::UnsupportedPlatform`] on Linux.
    /// - Otherwise the platform error, e.g. when a file does not exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri::{Runtime, WebviewWindow};
    /// use tauri_plugin_share::{ShareData, ShareExt, ShareOptions};
    ///
    /// fn share_link<R: Runtime>(window: &WebviewWindow<R>) -> tauri_plugin_share::Result<()> {
    ///     window.share().share(
    ///         window,
    ///         ShareData::new().text("Hello!").url("https://tauri.app"),
    ///         ShareOptions::new(),
    ///     )
    /// }
    /// ```
    pub fn share(
        &self,
        window: &WebviewWindow<R>,
        data: ShareData,
        options: ShareOptions,
    ) -> Result<()> {
        if data.is_empty() {
            return Err(Error::Empty);
        }
        self.0.share(window, data, options)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the share APIs.
pub trait ShareExt<R: Runtime> {
    /// Returns the [`Share`] instance managed by the app.
    fn share(&self) -> &Share<R>;
}

impl<R: Runtime, T: Manager<R>> crate::ShareExt<R> for T {
    fn share(&self) -> &Share<R> {
        self.state::<Share<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("share")
        .invoke_handler(tauri::generate_handler![commands::share])
        .setup(|app, api| {
            #[cfg(mobile)]
            let share = mobile::init(app, api)?;
            #[cfg(desktop)]
            let share = desktop::init(app, api)?;
            app.manage(Share(share));
            Ok(())
        })
        .build()
}
