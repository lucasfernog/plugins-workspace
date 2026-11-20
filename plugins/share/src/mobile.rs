// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime, WebviewWindow,
    plugin::{PluginApi, PluginHandle},
};

use crate::{Rect, ShareData, ShareOptions};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.share";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_share);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Share<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "SharePlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_share)?;
    Ok(Share(handle))
}

#[derive(Serialize)]
struct SharePayload {
    #[serde(flatten)]
    data: ShareData,
    #[serde(skip_serializing_if = "Option::is_none")]
    anchor: Option<Rect>,
}

pub struct Share<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Share<R> {
    pub fn share(
        &self,
        _window: &WebviewWindow<R>,
        data: ShareData,
        options: ShareOptions,
    ) -> crate::Result<()> {
        self.0
            .run_mobile_plugin(
                "share",
                SharePayload {
                    data,
                    anchor: options.anchor,
                },
            )
            .map_err(Into::into)
    }
}
