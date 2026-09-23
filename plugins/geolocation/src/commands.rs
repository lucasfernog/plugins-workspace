// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, ipc::Channel, AppHandle, Runtime};

// These commands are async and use the non-blocking plugin calls: a location or permission request
// can take as long as the user leaves the system dialog open, and must not block a runtime thread.
use crate::{GeolocationExt, PermissionStatus, PermissionType, Position, PositionOptions, Result};

#[command]
pub(crate) async fn get_current_position<R: Runtime>(
    app: AppHandle<R>,
    options: Option<PositionOptions>,
) -> Result<Position> {
    app.geolocation().get_current_position_async(options).await
}

#[command]
pub(crate) async fn watch_position<R: Runtime>(
    app: AppHandle<R>,
    options: PositionOptions,
    channel: Channel,
) -> Result<()> {
    app.geolocation()
        .watch_position_async(options, channel)
        .await
}

#[command]
pub(crate) async fn clear_watch<R: Runtime>(app: AppHandle<R>, channel_id: u32) -> Result<()> {
    app.geolocation().clear_watch_async(channel_id).await
}

#[command]
pub(crate) async fn check_permissions<R: Runtime>(app: AppHandle<R>) -> Result<PermissionStatus> {
    app.geolocation().check_permissions_async().await
}

#[command]
pub(crate) async fn request_permissions<R: Runtime>(
    app: AppHandle<R>,
    permissions: Option<Vec<PermissionType>>,
) -> Result<PermissionStatus> {
    app.geolocation()
        .request_permissions_async(permissions)
        .await
}
