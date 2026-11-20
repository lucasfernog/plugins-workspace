// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Schedule background tasks written in Rust on Android and iOS.
//!
//! Tasks are async Rust functions registered by name in a [`Tasks`] set. The system runs
//! them while the app is in the background, through
//! [WorkManager](https://developer.android.com/develop/background-work/background-tasks/persistent)
//! on Android and
//! [BGTaskScheduler](https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler)
//! on iOS.
//!
//! ```
//! use std::time::Duration;
//! use tauri_plugin_background_tasks::{
//!     BackgroundTasksExt, Outcome, TaskContext, TaskRequest, Tasks,
//! };
//!
//! async fn sync(ctx: TaskContext) -> Outcome {
//!     // do the work, checking `ctx.is_cancelled()` between steps
//!     Outcome::Success
//! }
//!
//! fn tasks() -> Tasks {
//!     Tasks::new().task("com.example.app.sync", sync)
//! }
//!
//! // lets Android run the tasks when the app is not running
//! tauri_plugin_background_tasks::export_tasks!(tasks);
//!
//! fn builder<R: tauri::Runtime>() -> tauri::Builder<R> {
//!     tauri::Builder::new()
//!         .plugin(tauri_plugin_background_tasks::init(tasks()))
//!         .setup(|app| {
//!             app.background_tasks().schedule(
//!                 TaskRequest::new("com.example.app.sync")
//!                     .interval(Duration::from_secs(60 * 60))
//!                     .requires_network(true),
//!             )?;
//!             Ok(())
//!         })
//! }
//! ```
//!
//! ## Where a task runs
//!
//! - **iOS** launches the whole app in the background, so [`TaskContext::app_handle`]
//!   returns the app.
//! - **Android** runs the task in the app's process. If the process is not alive, it is
//!   started without an activity: the Tauri app is not created, the handler is reached
//!   through the symbols [`export_tasks!`] exports and [`TaskContext::app_handle`]
//!   returns `None`. Handlers must not rely on app state or on Android APIs that need an
//!   activity.

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

#[cfg(target_os = "android")]
mod android;
mod commands;
mod error;
mod models;
mod task;

pub use error::{Error, Result};
pub use task::{Outcome, TaskContext, Tasks};

#[cfg(desktop)]
pub use desktop::BackgroundTasks;
#[cfg(mobile)]
pub use mobile::BackgroundTasks;

#[doc(hidden)]
pub mod __private {
    #[cfg(target_os = "android")]
    pub use crate::android::{cancel, run};
}

/// Exports the entry points Android's WorkManager uses to run the tasks of the app.
///
/// Call it once, in the app's library crate, with the path of a function that returns
/// the app's [`Tasks`]. Without it Android can't find the task handlers and every task
/// fails. It expands to nothing on other platforms.
///
/// ```
/// use tauri_plugin_background_tasks::Tasks;
///
/// fn tasks() -> Tasks {
///     Tasks::new().task("com.example.app.cleanup", |_ctx| async {})
/// }
///
/// tauri_plugin_background_tasks::export_tasks!(tasks);
/// ```
#[macro_export]
macro_rules! export_tasks {
    ($tasks:path) => {
        #[cfg(target_os = "android")]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn Java_app_tauri_backgroundtasks_TaskRunner_run(
            env: *mut ::core::ffi::c_void,
            _class: *mut ::core::ffi::c_void,
            run_id: *mut ::core::ffi::c_void,
            name: *mut ::core::ffi::c_void,
            input: *mut ::core::ffi::c_void,
        ) -> i32 {
            unsafe { $crate::__private::run(env, run_id, name, input, $tasks) }
        }

        #[cfg(target_os = "android")]
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn Java_app_tauri_backgroundtasks_TaskRunner_cancel(
            env: *mut ::core::ffi::c_void,
            _class: *mut ::core::ffi::c_void,
            run_id: *mut ::core::ffi::c_void,
        ) {
            unsafe { $crate::__private::cancel(env, run_id) }
        }
    };
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the background tasks APIs.
pub trait BackgroundTasksExt<R: Runtime> {
    /// Returns the handle to the background tasks APIs.
    fn background_tasks(&self) -> &BackgroundTasks<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BackgroundTasksExt<R> for T {
    fn background_tasks(&self) -> &BackgroundTasks<R> {
        self.state::<BackgroundTasks<R>>().inner()
    }
}

/// Initializes the plugin with the task handlers of the app.
///
/// On Android, also call [`export_tasks!`] so the tasks can run when the app is not.
pub fn init<R: Runtime>(tasks: Tasks) -> TauriPlugin<R> {
    Builder::new("background-tasks")
        .invoke_handler(tauri::generate_handler![
            commands::schedule,
            commands::cancel,
            commands::cancel_all,
            commands::get_scheduled
        ])
        .setup(move |app, api| {
            task::set_app_handle(app.clone());
            #[cfg(mobile)]
            let background_tasks = mobile::init(app, api, tasks)?;
            #[cfg(desktop)]
            let background_tasks = desktop::init(app, api, tasks)?;
            app.manage(background_tasks);
            Ok(())
        })
        .build()
}
