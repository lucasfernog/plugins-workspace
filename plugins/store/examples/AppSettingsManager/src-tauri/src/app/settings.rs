// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_plugin_store::Store;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub theme: String,
}

impl<R: tauri::Runtime> From<&Store<R>> for AppSettings {
    fn from(store: &Store<R>) -> Self {
        // `main.rs` saves the settings as one object under the `appSettings` key
        let app_settings = store.get("appSettings");

        let launch_at_login = app_settings
            .as_ref()
            .and_then(|settings| settings.get("launchAtLogin"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let theme = app_settings
            .as_ref()
            .and_then(|settings| settings.get("theme"))
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "dark".to_owned());

        AppSettings {
            launch_at_login,
            theme,
        }
    }
}
