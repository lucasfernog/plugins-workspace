// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use arboard::ImageData;
use serde::de::DeserializeOwned;
use tauri::{image::Image, plugin::PluginApi, AppHandle, Runtime};

use std::{borrow::Cow, sync::Mutex};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    Ok(Clipboard {
        app: app.clone(),
        clipboard: arboard::Clipboard::new().map(|c| Mutex::new(Some(c))),
    })
}

/// Checks that an RGBA buffer of `len` bytes holds exactly a `width` x `height` image.
///
/// arboard does not check this and panics (or reads out of bounds) on a mismatch.
fn validate_rgba_len(len: usize, width: u32, height: u32) -> crate::Result<()> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    match expected {
        Some(expected) if expected == len => Ok(()),
        Some(expected) => Err(crate::Error::Clipboard(format!(
            "invalid image: a {width}x{height} image needs {expected} bytes of RGBA data, got {len}"
        ))),
        None => Err(crate::Error::Clipboard(format!(
            "invalid image: a {width}x{height} image is too large"
        ))),
    }
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
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_text(text)
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Writes an image to the system clipboard as RGBA data.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the image's RGBA buffer is not exactly
    /// `width * height * 4` bytes long, the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn write_image(&self, image: &Image<'_>) -> crate::Result<()> {
        validate_rgba_len(image.rgba().len(), image.width(), image.height())?;
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_image(ImageData {
                    bytes: Cow::Borrowed(image.rgba()),
                    width: image.width() as usize,
                    height: image.height() as usize,
                })
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_text(&self) -> crate::Result<String> {
        match &self.clipboard {
            Ok(clipboard) => {
                let text = clipboard.lock().unwrap().as_mut().unwrap().get_text()?;
                Ok(text)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
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
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_html(html, alt_text)
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Clears the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::Clipboard`] if the clipboard could not be initialized or the
    /// underlying [`arboard`] operation fails.
    pub fn clear(&self) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .clear()
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_image(&self) -> crate::Result<Image<'_>> {
        match &self.clipboard {
            Ok(clipboard) => {
                let image = clipboard.lock().unwrap().as_mut().unwrap().get_image()?;
                let image = Image::new_owned(
                    image.bytes.to_vec(),
                    image.width as u32,
                    image.height as u32,
                );
                Ok(image)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub(crate) fn cleanup(&self) {
        if let Ok(clipboard) = &self.clipboard {
            clipboard.lock().unwrap().take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::validate_rgba_len;

    #[test]
    fn rgba_len_matches() {
        assert!(validate_rgba_len(16, 2, 2).is_ok());
        assert!(validate_rgba_len(4, 1, 1).is_ok());
        assert!(validate_rgba_len(0, 0, 0).is_ok());
    }

    #[test]
    fn rgba_len_mismatch() {
        let err = validate_rgba_len(8, 2, 2).unwrap_err().to_string();
        assert!(err.contains("needs 16 bytes"), "{err}");
        assert!(validate_rgba_len(20, 2, 2).is_err());
        assert!(validate_rgba_len(8, 0, 2).is_err());
    }

    #[test]
    fn rgba_len_overflow() {
        // u32::MAX * u32::MAX * 4 overflows even a 64-bit usize
        let err = validate_rgba_len(0, u32::MAX, u32::MAX)
            .unwrap_err()
            .to_string();
        assert!(err.contains("too large"), "{err}");
        assert!(validate_rgba_len(0, u32::MAX, 2).is_err());
    }
}
