// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, WebviewWindow, plugin::PluginApi};

use crate::{ShareData, ShareOptions};

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Share<R>> {
    Ok(Share(PhantomData))
}

pub struct Share<R: Runtime>(PhantomData<fn() -> R>);

impl<R: Runtime> Share<R> {
    pub fn share(
        &self,
        window: &WebviewWindow<R>,
        data: ShareData,
        options: ShareOptions,
    ) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        return crate::macos::share(window, data, options);
        #[cfg(windows)]
        return crate::windows::share(window, data, options);
        #[cfg(not(any(target_os = "macos", windows)))]
        {
            let _ = (window, data, options);
            Err(crate::Error::UnsupportedPlatform)
        }
    }
}

/// Runs `f` on the main thread and waits for its result.
#[cfg(any(target_os = "macos", windows))]
pub(crate) fn run_on_main_thread<R: Runtime, T: Send + 'static>(
    window: &WebviewWindow<R>,
    f: impl FnOnce(&WebviewWindow<R>) -> crate::Result<T> + Send + 'static,
) -> crate::Result<T> {
    let (tx, rx) = std::sync::mpsc::channel();
    let window_ = window.clone();
    window.run_on_main_thread(move || {
        let _ = tx.send(f(&window_));
    })?;
    rx.recv().map_err(|_| crate::Error::MainThread)?
}
