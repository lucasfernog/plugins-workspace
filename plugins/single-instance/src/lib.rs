// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Ensure a single instance of your tauri app is running.
//!
//! When a second instance starts, it sends its command line arguments and working directory to
//! the first instance, which runs the callback passed to [`init`] (or [`Builder::callback`]),
//! and exits. See [`init`] for caveats.
//!
//! How instances find each other:
//!
//! - **Windows**: a named mutex and a hidden message window, both named after the bundle
//!   identifier.
//! - **Linux**: the D-Bus session bus name `<identifier>.SingleInstance` (see
//!   [`Builder::dbus_id`]). Without a session bus the app runs without single-instance
//!   protection.
//! - **macOS**: a Unix socket named after the identifier in the per-user temporary directory.
//!
//! ## Cargo features
//!
//! - **semver**: Allows the app with SemVer incompatible versions to run alongside each other.
//!   It appends the SemVer-compatible part of the version to the mutex, D-Bus and socket names.
//! - **deep-link**: Trigger [`tauri-plugin-deep-link`](https://crates.io/crates/tauri-plugin-deep-link) event before invoking the single-instance callback.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Runtime};

#[cfg(target_os = "windows")]
#[path = "platform_impl/windows.rs"]
mod platform_impl;
#[cfg(target_os = "linux")]
#[path = "platform_impl/linux.rs"]
mod platform_impl;
#[cfg(target_os = "macos")]
#[path = "platform_impl/macos.rs"]
mod platform_impl;

#[cfg(feature = "semver")]
mod semver_compat;

pub(crate) type SingleInstanceCallback<R> =
    dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static;

/// Initializes the plugin, calling `f` whenever a second instance of the app is started.
///
/// This is a shortcut for [`Builder::new`] with [`Builder::callback`] set to `f`, then
/// [`Builder::build`]. Use [`Builder`] directly if you need to set a custom [`Builder::dbus_id`].
///
/// `f` is called with the app handle, the second instance's command line arguments
/// (as collected by [`std::env::args_os`], converted lossily to UTF-8, so the first element is
/// the executable path) and its current working directory. If the `deep-link` feature is
/// enabled, the arguments are first forwarded to
/// [`tauri-plugin-deep-link`](https://crates.io/crates/tauri-plugin-deep-link) before `f` runs.
///
/// The second instance never reaches [`tauri::Builder::run`]: it hands its arguments and working
/// directory off to the first instance and exits immediately.
///
/// # Caveats
///
/// - **Treat the arguments as untrusted input.** Any process running as the same user can
///   deliver arguments to `f` through the plugin's IPC channel (a window message on Windows,
///   D-Bus on Linux, a Unix socket on macOS). Validate them before acting on them, and don't
///   forward them to the frontend unfiltered.
/// - `f` runs on a different thread depending on the platform: the main thread on Windows, a
///   D-Bus executor thread on Linux and an async runtime worker on macOS. Use
///   [`AppHandle::run_on_main_thread`] for work that must happen on the main thread, and keep `f`
///   short since it blocks the delivery of later notifications.
/// - On Windows, a second instance running at a lower integrity level than the first one (for
///   example not elevated, while the first instance runs as administrator) can't reach it: its
///   arguments are dropped and it exits.
/// - Register the plugin on the [`tauri::Builder`] before other plugins, not from a `setup`
///   hook: windows declared in `tauri.conf.json` are created before `setup` runs, so a second
///   instance would briefly show them.
/// - The crate is empty on Android and iOS, so gate the call with `#[cfg(desktop)]`.
pub fn init<R: Runtime, F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>(
    f: F,
) -> TauriPlugin<R> {
    Builder::new().callback(f).build()
}

/// Releases the resources this plugin uses to detect other instances (the named mutex on
/// Windows, the D-Bus name on Linux or the Unix socket on macOS).
///
/// The plugin calls this automatically on [`tauri::RunEvent::Exit`], so you normally don't need
/// to call it yourself. Call it manually before terminating the process through means that skip
/// that event, such as [`std::process::exit`], so a future instance of the app isn't mistaken
/// for a still-running one.
pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    platform_impl::destroy(manager)
}

/// Builds the single-instance plugin.
///
/// Created with [`Builder::new`] and consumed by [`Builder::build`].
pub struct Builder<R: Runtime> {
    callback: Box<SingleInstanceCallback<R>>,
    dbus_id: Option<String>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            callback: Box::new(move |_app, _args, _| {
                #[cfg(feature = "deep-link")]
                if let Some(deep_link) = _app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
                    deep_link.handle_cli_arguments(_args.iter());
                }
            }),
            dbus_id: None,
        }
    }
}

impl<R: Runtime> Builder<R> {
    /// Creates a new builder with a no-op callback (or, when the `deep-link` feature is enabled,
    /// a callback that only forwards the arguments to the deep-link plugin) and no custom D-Bus
    /// ID. Use [`Builder::callback`] and [`Builder::dbus_id`] to configure it, then
    /// [`Builder::build`] to create the plugin.
    pub fn new() -> Self {
        Default::default()
    }

    /// Function to call when a secondary instance was opened by the user and killed by the plugin.
    /// If the `deep-link` feature is enabled, the plugin triggers the deep-link plugin before executing the callback.
    ///
    /// See [`init`] for the callback's arguments and caveats.
    pub fn callback<F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>(
        mut self,
        mut f: F,
    ) -> Self {
        self.callback = Box::new(move |app, args, cwd| {
            #[cfg(feature = "deep-link")]
            if let Some(deep_link) = app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
                deep_link.handle_cli_arguments(args.iter());
            }
            f(app, args, cwd)
        });
        self
    }

    /// Set a custom D-Bus ID, used on Linux. The plugin will append a `.SingleInstance` subname.
    /// For example `com.mycompany.myapp` will result in the plugin registering its D-Bus service on `com.mycompany.myapp.SingleInstance`.
    /// Usually you want the same base ID across all components in your app.
    ///
    /// Defaults to the app's bundle identifier set in tauri.conf.json. With the `semver` feature,
    /// a version suffix is appended after `.SingleInstance`, e.g. `_1_x_x`. Characters that are
    /// not valid in a D-Bus name are replaced with `_`.
    ///
    /// Set this when your Flatpak or Snap app ID differs from the Tauri identifier, since the
    /// sandbox only lets the app own names under its app ID. Versions of this plugin before
    /// 2.4.0 used `org.<identifier with . and - replaced by _>.SingleInstance`.
    pub fn dbus_id(mut self, dbus_id: impl Into<String>) -> Self {
        self.dbus_id = Some(dbus_id.into());
        self
    }

    /// Builds the single-instance [`TauriPlugin`].
    ///
    /// Register it first among your app's plugins, since the plugins run in the order they were
    /// added and a second instance must be detected before the rest of your setup runs.
    pub fn build(self) -> TauriPlugin<R> {
        platform_impl::init(
            self.callback,
            #[cfg(target_os = "linux")]
            self.dbus_id,
        )
    }
}
