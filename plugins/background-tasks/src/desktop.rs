// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

use crate::{Error, ScheduledTask, TaskRequest, Tasks};

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
    _tasks: Tasks,
) -> crate::Result<BackgroundTasks<R>> {
    Ok(BackgroundTasks(PhantomData))
}

/// Access to the background tasks APIs.
///
/// Background tasks are not available on desktop: every method returns
/// [`Error::Unsupported`].
pub struct BackgroundTasks<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> BackgroundTasks<R> {
    /// Schedules a task, replacing the pending request with the same name.
    pub fn schedule(&self, _request: TaskRequest) -> crate::Result<()> {
        Err(Error::Unsupported)
    }

    /// Cancels the pending request of the task called `name`.
    pub fn cancel(&self, _name: &str) -> crate::Result<()> {
        Err(Error::Unsupported)
    }

    /// Cancels every pending request scheduled through this plugin.
    pub fn cancel_all(&self) -> crate::Result<()> {
        Err(Error::Unsupported)
    }

    /// Returns the tasks that are waiting to run.
    pub fn scheduled(&self) -> crate::Result<Vec<ScheduledTask>> {
        Err(Error::Unsupported)
    }
}
