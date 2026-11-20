// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime, command};

use crate::{BackgroundTasksExt, Result, ScheduledTask, TaskRequest};

#[command]
pub(crate) async fn schedule<R: Runtime>(app: AppHandle<R>, request: TaskRequest) -> Result<()> {
    app.background_tasks().schedule(request)
}

#[command]
pub(crate) async fn cancel<R: Runtime>(app: AppHandle<R>, name: String) -> Result<()> {
    app.background_tasks().cancel(&name)
}

#[command]
pub(crate) async fn cancel_all<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.background_tasks().cancel_all()
}

#[command]
pub(crate) async fn get_scheduled<R: Runtime>(app: AppHandle<R>) -> Result<Vec<ScheduledTask>> {
    app.background_tasks().scheduled()
}
