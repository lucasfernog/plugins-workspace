// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The content to share. At least one of [`text`](Self::text), [`url`](Self::url) and
/// [`files`](Self::files) must be set.
///
/// # Examples
///
/// ```
/// use tauri_plugin_share::ShareData;
///
/// let data = ShareData::new()
///     .title("Tauri")
///     .text("Build smaller, faster and more secure apps")
///     .url("https://tauri.app");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShareData {
    /// A title for the shared content.
    ///
    /// - **Android:** shown at the top of the share sheet and used as the subject by apps that
    ///   have one, such as email clients.
    /// - **iOS:** used as the subject by apps that have one.
    /// - **Windows:** shown in the share dialog. Defaults to the app name.
    /// - **macOS:** ignored.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// The text to share.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// The URL to share.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// The files to share, as file system paths.
    ///
    /// On Android the files must be inside one of the app's directories (data, cache and their
    /// external storage counterparts), or be `content://` URIs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<PathBuf>,
}

impl ShareData {
    /// Creates an empty [`ShareData`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title of the shared content. See [`Self::title`].
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title.replace(title.into());
        self
    }

    /// Sets the text to share.
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text.replace(text.into());
        self
    }

    /// Sets the URL to share.
    #[must_use]
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url.replace(url.into());
        self
    }

    /// Adds a file to share. See [`Self::files`].
    #[must_use]
    pub fn file(mut self, path: impl Into<PathBuf>) -> Self {
        self.files.push(path.into());
        self
    }

    /// Adds files to share. See [`Self::files`].
    #[must_use]
    pub fn files<P: Into<PathBuf>>(mut self, paths: impl IntoIterator<Item = P>) -> Self {
        self.files.extend(paths.into_iter().map(Into::into));
        self
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.text.is_none() && self.url.is_none() && self.files.is_empty()
    }
}

/// A rectangle in logical pixels, relative to the top-left corner of the window's content area.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    /// The horizontal offset of the rectangle.
    pub x: f64,
    /// The vertical offset of the rectangle.
    pub y: f64,
    /// The width of the rectangle.
    #[serde(default)]
    pub width: f64,
    /// The height of the rectangle.
    #[serde(default)]
    pub height: f64,
}

/// Options for how the share sheet is presented.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShareOptions {
    /// The area the share sheet is anchored to, usually the button that triggered it.
    ///
    /// Used on macOS and on iPad, where the share sheet is a popover. Defaults to the center of
    /// the window. Ignored on the other platforms.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Rect>,
}

impl ShareOptions {
    /// Creates the default [`ShareOptions`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the area the share sheet is anchored to. See [`Self::anchor`].
    #[must_use]
    pub fn anchor(mut self, anchor: Rect) -> Self {
        self.anchor.replace(anchor);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_data_from_js() {
        let data: ShareData = serde_json::from_value(serde_json::json!({
            "title": "Tauri",
            "url": "https://tauri.app",
            "files": ["/tmp/a.txt"]
        }))
        .unwrap();
        assert_eq!(
            data,
            ShareData::new()
                .title("Tauri")
                .url("https://tauri.app")
                .file("/tmp/a.txt")
        );
    }

    #[test]
    fn title_alone_is_empty() {
        assert!(ShareData::new().is_empty());
        assert!(ShareData::new().title("Tauri").is_empty());
        assert!(!ShareData::new().text("hello").is_empty());
        assert!(!ShareData::new().url("https://tauri.app").is_empty());
        assert!(!ShareData::new().file("/tmp/a.txt").is_empty());
    }

    #[test]
    fn unset_fields_are_not_sent_to_mobile() {
        assert_eq!(
            serde_json::to_value(ShareData::new().text("hello")).unwrap(),
            serde_json::json!({ "text": "hello" })
        );
    }

    #[test]
    fn anchor_size_is_optional() {
        let options: ShareOptions =
            serde_json::from_value(serde_json::json!({ "anchor": { "x": 1, "y": 2 } })).unwrap();
        assert_eq!(
            options.anchor,
            Some(Rect {
                x: 1.,
                y: 2.,
                width: 0.,
                height: 0.
            })
        );
    }
}
