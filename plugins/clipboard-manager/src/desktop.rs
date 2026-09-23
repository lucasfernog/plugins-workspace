// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use arboard::ImageData;
use serde::de::DeserializeOwned;
use tauri::{image::Image, plugin::PluginApi, AppHandle, Runtime};

use std::{
    borrow::Cow,
    sync::{Mutex, PoisonError},
};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    Ok(Clipboard {
        app: app.clone(),
        clipboard: arboard::Clipboard::new().map(|c| Mutex::new(Some(c))),
    })
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    // According to arboard docs the clipboard must be dropped before exit.
    // Since tauri doesn't call drop on exit we'll use an Option to take() on RunEvent::Exit.
    clipboard: Result<Mutex<Option<arboard::Clipboard>>, arboard::Error>,
}

impl<R: Runtime> Clipboard<R> {
    /// Writes plain text to the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn write_text<'a, T: Into<Cow<'a, str>>>(&self, text: T) -> crate::Result<()> {
        self.with_clipboard(|clipboard| clipboard.set_text(text))
    }

    /// Writes an image to the system clipboard as RGBA data.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn write_image(&self, image: &Image<'_>) -> crate::Result<()> {
        self.with_clipboard(|clipboard| {
            clipboard.set_image(ImageData {
                bytes: Cow::Borrowed(image.rgba()),
                width: image.width() as usize,
                height: image.height() as usize,
            })
        })
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_text(&self) -> crate::Result<String> {
        self.with_clipboard(|clipboard| clipboard.get_text())
    }

    /// Writes HTML to the system clipboard, with an optional plain text fallback for targets
    /// that cannot render it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn write_html<'a, T: Into<Cow<'a, str>>>(
        &self,
        html: T,
        alt_text: Option<T>,
    ) -> crate::Result<()> {
        self.with_clipboard(|clipboard| clipboard.set_html(html, alt_text))
    }

    /// Clears the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn clear(&self) -> crate::Result<()> {
        self.with_clipboard(|clipboard| clipboard.clear())
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_image(&self) -> crate::Result<Image<'_>> {
        let image = self.with_clipboard(|clipboard| clipboard.get_image())?;
        Ok(Image::new_owned(
            image.bytes.to_vec(),
            image.width as u32,
            image.height as u32,
        ))
    }

    /// Runs `f` with the inner [`arboard::Clipboard`].
    ///
    /// Never panics: a poisoned lock is recovered (the clipboard holds no invariant a panic could
    /// break), and using the clipboard after [`Self::cleanup`] returns an error.
    fn with_clipboard<T>(
        &self,
        f: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
    ) -> crate::Result<T> {
        match &self.clipboard {
            Ok(clipboard) => {
                let mut clipboard = clipboard.lock().unwrap_or_else(PoisonError::into_inner);
                let clipboard = clipboard.as_mut().ok_or_else(|| {
                    crate::Error::Clipboard("the clipboard has already been closed".into())
                })?;
                f(clipboard).map_err(Into::into)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub(crate) fn cleanup(&self) {
        if let Ok(clipboard) = &self.clipboard {
            clipboard
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
        }
    }
}
