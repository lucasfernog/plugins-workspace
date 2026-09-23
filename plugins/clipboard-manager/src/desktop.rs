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
    let clipboard = match arboard::Clipboard::new() {
        Ok(clipboard) => Some(clipboard),
        Err(e) => {
            // e.g. the display server connection is not ready yet on Linux;
            // retried the next time the clipboard is used
            log::warn!("failed to initialize the clipboard, retrying on first use: {e}");
            None
        }
    };
    Ok(Clipboard {
        app: app.clone(),
        clipboard: Mutex::new(ClipboardState {
            clipboard,
            closed: false,
        }),
    })
}

struct ClipboardState {
    /// `None` until initialized successfully, and again once closed.
    clipboard: Option<arboard::Clipboard>,
    /// Set on `RunEvent::Exit`, after which the clipboard is never re-created.
    closed: bool,
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    // According to arboard docs the clipboard must be dropped before exit.
    // Since tauri doesn't call drop on exit we'll use an Option to take() on RunEvent::Exit.
    clipboard: Mutex<ClipboardState>,
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
    /// If the clipboard failed to initialize, initializing it is retried.
    fn with_clipboard<T>(
        &self,
        f: impl FnOnce(&mut arboard::Clipboard) -> Result<T, arboard::Error>,
    ) -> crate::Result<T> {
        let mut state = self
            .clipboard
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if state.closed {
            return Err(crate::Error::Clipboard(
                "the clipboard has already been closed".into(),
            ));
        }
        let clipboard = match &mut state.clipboard {
            Some(clipboard) => clipboard,
            clipboard @ None => clipboard.insert(arboard::Clipboard::new()?),
        };
        f(clipboard).map_err(Into::into)
    }

    pub(crate) fn cleanup(&self) {
        let mut state = self
            .clipboard
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        state.closed = true;
        state.clipboard.take();
    }
}
