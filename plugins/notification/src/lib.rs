// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Send message notifications (brief auto-expiring OS window element) to your user. Can also be used with the Notification Web API.
//!
//! ## Cargo features
//!
//! - **windows7-compat**: Adds support for the legacy Windows 7 notification implementation and Windows-version detection.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use serde::Serialize;
#[cfg(mobile)]
use tauri::plugin::PluginHandle;
#[cfg(desktop)]
use tauri::AppHandle;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;
pub use tauri::plugin::PermissionState;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::Notification;
#[cfg(mobile)]
pub use mobile::Notification;

/// The notification builder.
#[derive(Debug)]
pub struct NotificationBuilder<R: Runtime> {
    #[cfg(desktop)]
    app: AppHandle<R>,
    #[cfg(mobile)]
    handle: PluginHandle<R>,
    pub(crate) data: NotificationData,
}

impl<R: Runtime> NotificationBuilder<R> {
    #[cfg(desktop)]
    fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            data: Default::default(),
        }
    }

    #[cfg(mobile)]
    fn new(handle: PluginHandle<R>) -> Self {
        Self {
            handle,
            data: Default::default(),
        }
    }

    /// Sets the notification identifier.
    pub fn id(mut self, id: i32) -> Self {
        self.data.id = id;
        self
    }

    /// Sets the identifier of the notification channel that delivers this notification.
    ///
    /// If the channel does not exist, the notification won't fire.
    /// Make sure the channel exists with `Notification::list_channels` and
    /// `Notification::create_channel`.
    ///
    /// Only used on Android.
    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.data.channel_id.replace(id.into());
        self
    }

    /// Sets the notification title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.data.title.replace(title.into());
        self
    }

    /// Sets the notification body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.data.body.replace(body.into());
        self
    }

    /// Schedule this notification to fire on a later time or a fixed interval.
    pub fn schedule(mut self, schedule: Schedule) -> Self {
        self.data.schedule.replace(schedule);
        self
    }

    /// Multiline text.
    /// Changes the notification style to big text.
    /// Cannot be used with `inboxLines`.
    ///
    /// Only used on Android.
    pub fn large_body(mut self, large_body: impl Into<String>) -> Self {
        self.data.large_body.replace(large_body.into());
        self
    }

    /// Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
    ///
    /// Only used on Android.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.data.summary.replace(summary.into());
        self
    }

    /// Sets the identifier of an action type registered with
    /// `Notification::register_action_types`, whose actions are displayed on this notification.
    ///
    /// Only used on mobile.
    pub fn action_type_id(mut self, action_type_id: impl Into<String>) -> Self {
        self.data.action_type_id.replace(action_type_id.into());
        self
    }

    /// Identifier used to group multiple notifications.
    ///
    /// <https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier>
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.data.group.replace(group.into());
        self
    }

    /// Instructs the system that this notification is the summary of a group.
    ///
    /// Only used on Android.
    pub fn group_summary(mut self) -> Self {
        self.data.group_summary = true;
        self
    }

    /// The sound for the notification.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS**: a system sound name (e.g. `Ping`) or a sound file in the app bundle.
    /// - **Linux**: an XDG sound theme name (e.g. `message-new-instant`).
    /// - **Windows**: one of the built-in toast sounds (`Default`, `IM`, `Mail`, `Reminder`, `SMS`,
    ///   `Alarm`, `Alarm2`-`Alarm10`, `Call`, `Call2`-`Call10`). File paths are not supported.
    /// - **Android**: the name of a sound resource in the app's `res/raw` folder. On Android 8+ the
    ///   notification channel decides the sound.
    /// - **iOS**: the name of a sound file in the app bundle.
    pub fn sound(mut self, sound: impl Into<String>) -> Self {
        self.data.sound.replace(sound.into());
        self
    }

    /// Append an inbox line to the notification.
    /// Changes the notification style to inbox.
    /// Cannot be used with `largeBody`.
    ///
    /// Only supports up to 5 lines. Only used on Android.
    pub fn inbox_line(mut self, line: impl Into<String>) -> Self {
        self.data.inbox_lines.push(line.into());
        self
    }

    /// Notification icon.
    ///
    /// On Android the icon must be placed in the app's `res/drawable` folder.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.data.icon.replace(icon.into());
        self
    }

    /// Notification large icon. Only used on Android.
    ///
    /// The icon must be placed in the app's `res/drawable` folder.
    pub fn large_icon(mut self, large_icon: impl Into<String>) -> Self {
        self.data.large_icon.replace(large_icon.into());
        self
    }

    /// Icon color, as a hex color string such as `#ff0000`. Only used on Android.
    pub fn icon_color(mut self, icon_color: impl Into<String>) -> Self {
        self.data.icon_color.replace(icon_color.into());
        self
    }

    /// Append an attachment to the notification. Only used on iOS.
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.data.attachments.push(attachment);
        self
    }

    /// Adds an extra payload to store in the notification.
    pub fn extra(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.data
            .extra
            .insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }

    /// If true, the notification cannot be dismissed by the user on Android.
    ///
    /// An application service must manage the dismissal of the notification.
    /// It is typically used to indicate a background task that is pending (e.g. a file download)
    /// or the user is engaged with (e.g. playing music).
    pub fn ongoing(mut self) -> Self {
        self.data.ongoing = true;
        self
    }

    /// Automatically cancel the notification when the user clicks on it. Only used on Android.
    pub fn auto_cancel(mut self) -> Self {
        self.data.auto_cancel = true;
        self
    }

    /// Presents the notification without a banner, sound or badge while the app is in the foreground.
    ///
    /// Only used on iOS; notifications delivered while the app is in the background are not affected.
    pub fn silent(mut self) -> Self {
        self.data.silent = true;
        self
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the notification APIs.
pub trait NotificationExt<R: Runtime> {
    /// Returns the notification APIs managed by the plugin.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_notification::NotificationExt;
    ///
    /// fn notify<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    ///   app.notification()
    ///     .builder()
    ///     .title("Tauri")
    ///     .body("Tauri is awesome!")
    ///     .show()
    ///     .unwrap();
    /// }
    /// ```
    fn notification(&self) -> &Notification<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NotificationExt<R> for T {
    fn notification(&self) -> &Notification<R> {
        self.state::<Notification<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("notification")
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::request_permission,
            commands::is_permission_granted
        ])
        .js_init_script(include_str!("init-iife.js").replace(
            "__TEMPLATE_windows__",
            if cfg!(windows) { "true" } else { "false" },
        ))
        .setup(|app, api| {
            #[cfg(mobile)]
            let notification = mobile::init(app, api)?;
            #[cfg(desktop)]
            let notification = desktop::init(app, api)?;
            app.manage(notification);
            Ok(())
        })
        .build()
}
