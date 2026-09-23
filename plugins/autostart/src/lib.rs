// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Automatically launch your application at startup. Supports Windows, Mac (via AppleScript or Launch Agent), and Linux.
//!
//! This crate is empty on Android and iOS, so gate every use of it (and its dependency in
//! `Cargo.toml`) to desktop targets, for example with `#[cfg(desktop)]`:
//!
//! ```no_run
//! # fn run<R: tauri::Runtime>(builder: tauri::Builder<R>) {
//! builder.setup(|app| {
//!     #[cfg(desktop)]
//!     app.handle().plugin(
//!         tauri_plugin_autostart::Builder::new()
//!             .arg("--autostarted")
//!             .build(),
//!     )?;
//!     Ok(())
//! });
//! # }
//! ```

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use auto_launch::{AutoLaunch, AutoLaunchBuilder};
use serde::{ser::Serializer, Serialize};
use tauri::{
    command,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};

use std::env::current_exe;

type Result<T> = std::result::Result<T, Error>;

/// The strategy used to register the application for auto start on macOS.
///
/// The builder's default is [`MacosLauncher::LaunchAgent`].
#[derive(Debug, Default, Copy, Clone)]
pub enum MacosLauncher {
    /// Auto start by installing a Launch Agent plist under `~/Library/LaunchAgents`,
    /// which macOS starts automatically at login.
    ///
    /// The plist is named after [`Builder::app_name`], which is also used as its `Label`, and
    /// starts the app's executable (`<App>.app/Contents/MacOS/<App>`) with [`Builder::args`].
    /// macOS 13+ shows a "Background Items Added" notification when it is enabled.
    #[default]
    LaunchAgent,
    /// Auto start by adding a login item through an AppleScript command sent to the
    /// "System Events" application.
    ///
    /// - The login item opens the `.app` bundle, and is named after the bundle's file name:
    ///   [`Builder::app_name`] is ignored.
    /// - Only the `--hidden` and `--minimized` [`Builder::args`] have an effect: either one
    ///   makes the login item hidden. Other arguments are ignored.
    /// - macOS asks the user for the Automation permission to control "System Events" the
    ///   first time autostart is enabled, disabled or checked. Set
    ///   `NSAppleEventsUsageDescription` in your app's `Info.plist` to explain why.
    AppleScript,
}

/// The error type of this plugin.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error, for example while resolving the current executable's path.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// An error forwarded from the underlying `auto_launch` operation, converted to its
    /// string representation.
    #[error("{0}")]
    Anyhow(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Manages the auto start (launch at login) state of the application.
///
/// An instance is created and managed as Tauri state when the plugin is built; access it
/// through [`ManagerExt::autolaunch`].
pub struct AutoLaunchManager(AutoLaunch);

impl AutoLaunchManager {
    /// Enables auto start, registering the application to launch at login.
    ///
    /// ## Errors
    ///
    /// Returns [`Error::Anyhow`] if the platform-specific registration fails, for example
    /// when the application path does not exist or is not absolute.
    pub fn enable(&self) -> Result<()> {
        self.0
            .enable()
            .map_err(|e| e.to_string())
            .map_err(Error::Anyhow)
    }

    /// Disables auto start, removing the application from the list of programs launched at login.
    ///
    /// ## Errors
    ///
    /// Returns [`Error::Anyhow`] if the platform-specific removal fails.
    pub fn disable(&self) -> Result<()> {
        match self.0.disable() {
            // On Windows, disabling deletes the app's `Run` registry value, which fails with
            // "not found" when autostart is already disabled. macOS and Linux treat that as a
            // no-op, so do the same here.
            Err(auto_launch::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result.map_err(|e| e.to_string()).map_err(Error::Anyhow),
        }
    }

    /// Returns whether auto start is currently enabled for the application.
    ///
    /// ## Errors
    ///
    /// Returns [`Error::Anyhow`] if the platform-specific check fails.
    pub fn is_enabled(&self) -> Result<bool> {
        self.0
            .is_enabled()
            .map_err(|e| e.to_string())
            .map_err(Error::Anyhow)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`]
/// and [`tauri::Window`] to access the autostart APIs.
pub trait ManagerExt<R: Runtime> {
    /// TODO: Rename these to `autostart` or `auto_start` in v3
    fn autolaunch(&self) -> State<'_, AutoLaunchManager>;
}

impl<R: Runtime, T: Manager<R>> ManagerExt<R> for T {
    /// TODO: Rename these to `autostart` or `auto_start` in v3
    fn autolaunch(&self) -> State<'_, AutoLaunchManager> {
        self.state::<AutoLaunchManager>()
    }
}

#[command]
async fn enable(manager: State<'_, AutoLaunchManager>) -> Result<()> {
    manager.enable()
}

#[command]
async fn disable(manager: State<'_, AutoLaunchManager>) -> Result<()> {
    manager.disable()
}

#[command]
async fn is_enabled(manager: State<'_, AutoLaunchManager>) -> Result<bool> {
    manager.is_enabled()
}

/// Builder for the autostart plugin, used to configure the startup arguments, application
/// name, and — on macOS — the launch strategy before calling [`Builder::build`].
#[derive(Default)]
pub struct Builder {
    #[cfg(target_os = "macos")]
    macos_launcher: MacosLauncher,
    args: Vec<String>,
    app_name: Option<String>,
}

impl Builder {
    /// Create a new auto start builder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument to pass to your app on startup.
    ///
    /// A common use is a flag that tells your app it was launched at login, which it can check
    /// with [`std::env::args`] (or the CLI plugin), for example to start minimized.
    ///
    /// With [`MacosLauncher::AppleScript`], only `--hidden` and `--minimized` have an effect.
    ///
    /// ## Examples
    ///
    /// ```
    /// tauri_plugin_autostart::Builder::new()
    ///     .arg("--from-autostart")
    ///     .arg("--hey");
    /// ```
    pub fn arg<S: Into<String>>(mut self, arg: S) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to pass to your app on startup.
    ///
    /// See [`Builder::arg`].
    ///
    /// ## Examples
    ///
    /// ```
    /// tauri_plugin_autostart::Builder::new().args(["--from-autostart", "--hey"]);
    /// ```
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }

    /// Sets whether to use launch agent or apple script to be used to enable auto start,
    /// the builder's default is [`MacosLauncher::LaunchAgent`]
    ///
    /// This method only exists on macOS, so gate the call with `#[cfg(target_os = "macos")]`.
    ///
    /// ## Examples
    ///
    /// ```
    /// let mut builder = tauri_plugin_autostart::Builder::new();
    /// #[cfg(target_os = "macos")]
    /// {
    ///     builder = builder.macos_launcher(tauri_plugin_autostart::MacosLauncher::AppleScript);
    /// }
    /// ```
    #[cfg(target_os = "macos")]
    pub fn macos_launcher(mut self, macos_launcher: MacosLauncher) -> Self {
        self.macos_launcher = macos_launcher;
        self
    }

    /// Sets the app name to be used for the auto start entry.
    ///
    /// Defaults to the app's `productName`. It names the Linux desktop entry
    /// (`~/.config/autostart/<name>.desktop`), the Windows `Run` registry value, and the macOS
    /// Launch Agent (`~/Library/LaunchAgents/<name>.plist`, also used as its `Label`). It is
    /// ignored with [`MacosLauncher::AppleScript`], which names the login item after the `.app`
    /// bundle.
    ///
    /// Two apps using the same name overwrite each other's entry, so consider using your
    /// bundle identifier. Changing the name (or the `productName`) in a later version leaves
    /// the entry created under the old name behind: disable autostart before renaming.
    ///
    /// ## Examples
    ///
    /// ```
    /// tauri_plugin_autostart::Builder::new().app_name("My Custom Name");
    /// ```
    pub fn app_name<S: Into<String>>(mut self, app_name: S) -> Self {
        self.app_name = Some(app_name.into());
        self
    }

    /// Builds the autostart [`TauriPlugin`] with the configured options.
    ///
    /// On setup, it resolves the current executable's path (or the AppImage path on Linux,
    /// when available) and manages an [`AutoLaunchManager`] built from it, so that
    /// [`ManagerExt::autolaunch`] can be used from anywhere in the application.
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        PluginBuilder::new("autostart")
            .invoke_handler(tauri::generate_handler![enable, disable, is_enabled])
            .setup(move |app, _api| {
                let mut builder = AutoLaunchBuilder::new();

                let app_name = self
                    .app_name
                    .as_ref()
                    .unwrap_or_else(|| &app.package_info().name);
                builder.set_app_name(app_name);

                builder.set_args(&self.args);

                let current_exe = current_exe()?;

                #[cfg(windows)]
                builder.set_app_path(&current_exe.display().to_string());

                #[cfg(target_os = "macos")]
                {
                    builder.set_use_launch_agent(matches!(
                        self.macos_launcher,
                        MacosLauncher::LaunchAgent
                    ));
                    // on macOS, current_exe gives path to /Applications/Example.app/MacOS/Example
                    // but this results in seeing a Unix Executable in macOS login items
                    // It must be: /Applications/Example.app
                    // If it didn't find exactly a single occurance of .app, it will default to
                    // exe path to not break it.
                    let exe_path = current_exe.canonicalize()?.display().to_string();
                    let parts: Vec<&str> = exe_path.split(".app/").collect();
                    let app_path = if parts.len() == 2
                        && matches!(self.macos_launcher, MacosLauncher::AppleScript)
                    {
                        format!("{}.app", parts.first().unwrap())
                    } else {
                        exe_path
                    };
                    builder.set_app_path(&app_path);
                }

                #[cfg(target_os = "linux")]
                if let Some(appimage) = app
                    .env()
                    .appimage
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                {
                    builder.set_app_path(&appimage);
                } else {
                    builder.set_app_path(&current_exe.display().to_string());
                }

                app.manage(AutoLaunchManager(
                    builder.build().map_err(|e| e.to_string())?,
                ));
                Ok(())
            })
            .build()
    }
}

/// Initializes the plugin.
///
/// `args` - are passed to your app on startup.
///
/// Prefer [`Builder`], which also accepts non-`'static` arguments and sets the entry's
/// [name](Builder::app_name). `init(launcher, args)` is the same as
/// `Builder::new().args(args).macos_launcher(launcher).build()`, where `launcher` is ignored
/// on platforms other than macOS.
pub fn init<R: Runtime>(
    #[allow(unused)] macos_launcher: MacosLauncher,
    args: Option<Vec<&'static str>>,
) -> TauriPlugin<R> {
    let mut builder = Builder::new();
    if let Some(args) = args {
        builder = builder.args(args)
    }
    #[cfg(target_os = "macos")]
    {
        builder = builder.macos_launcher(macos_launcher);
    }
    builder.build()
}
