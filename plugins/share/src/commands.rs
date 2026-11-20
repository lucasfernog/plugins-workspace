// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    Runtime, WebviewWindow,
    ipc::{CommandScope, GlobalScope},
};

use crate::{Error, ShareData, ShareExt, ShareOptions, scope};

#[tauri::command]
pub(crate) async fn share<R: Runtime>(
    window: WebviewWindow<R>,
    command_scope: CommandScope<scope::Entry>,
    global_scope: GlobalScope<scope::Entry>,
    data: ShareData,
    options: Option<ShareOptions>,
) -> crate::Result<()> {
    for file in &data.files {
        let allowed = scope::is_path_allowed(
            &window,
            command_scope.allows().iter().chain(global_scope.allows()),
            command_scope.denies().iter().chain(global_scope.denies()),
            file,
        )?;
        if !allowed {
            return Err(Error::ForbiddenPath(file.clone()));
        }
    }

    window
        .share()
        .share(&window, data, options.unwrap_or_default())
}
