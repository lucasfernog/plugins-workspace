// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::DeserializeOwned, Deserialize};
use tauri::{
    plugin::{mobile::PluginInvokeError, PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{FileDialogBuilder, FilePath, MessageDialogBuilder, MessageDialogResult};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.dialog";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_dialog);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Dialog<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "DialogPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_dialog)?;
    Ok(Dialog(handle))
}

/// Access to the dialog APIs.
#[derive(Debug)]
pub struct Dialog<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for Dialog<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> Dialog<R> {
    pub(crate) fn app_handle(&self) -> &AppHandle<R> {
        self.0.app()
    }
}

// iOS resolves a cancelled picker with `null`, and Android can resolve without a file,
// so both fields are optional instead of failing to deserialize.
#[derive(Debug, Deserialize)]
struct FilePickerResponse {
    #[serde(default)]
    files: Option<Vec<FilePath>>,
}

#[derive(Debug, Deserialize)]
struct SaveFileResponse {
    #[serde(default)]
    file: Option<FilePath>,
}

/// The rejection message of the Android file pickers when the user cancels them.
const ANDROID_PICKER_CANCELLED: &str = "File picker cancelled";

/// The dialog APIs report a failure the same way as a cancelled dialog, so log the error to
/// keep it observable.
fn log_error(command: &str, error: &PluginInvokeError) {
    if let PluginInvokeError::InvokeRejected(response) = error {
        if response.message.as_deref() == Some(ANDROID_PICKER_CANCELLED) {
            return;
        }
    }
    log::error!("dialog `{command}` failed, treating it as cancelled: {error}");
}

pub fn pick_file<R: Runtime, F: FnOnce(Option<FilePath>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<FilePickerResponse>("showFilePicker", dialog.payload(false));
        match res {
            // the native side can resolve with an empty list
            Ok(response) => f(response.files.and_then(|files| files.into_iter().next())),
            Err(e) => {
                log_error("showFilePicker", &e);
                f(None)
            }
        }
    });
}

pub fn pick_files<R: Runtime, F: FnOnce(Option<Vec<FilePath>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<FilePickerResponse>("showFilePicker", dialog.payload(true));
        match res {
            Ok(response) => f(response.files),
            Err(e) => {
                log_error("showFilePicker", &e);
                f(None)
            }
        }
    });
}

pub fn save_file<R: Runtime, F: FnOnce(Option<FilePath>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<SaveFileResponse>("saveFileDialog", dialog.payload(false));
        match res {
            Ok(response) => f(response.file),
            Err(e) => {
                log_error("saveFileDialog", &e);
                f(None)
            }
        }
    });
}

#[derive(Debug, Deserialize)]
struct ShowMessageDialogResponse {
    value: String,
}

/// Shows a message dialog
pub fn show_message_dialog<R: Runtime, F: FnOnce(MessageDialogResult) + Send + 'static>(
    dialog: MessageDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<ShowMessageDialogResponse>("showMessageDialog", dialog.payload());

        match res {
            Ok(res) => f(res.value.into()),
            Err(e) => {
                log_error("showMessageDialog", &e);
                f(MessageDialogResult::default())
            }
        }
    });
}
