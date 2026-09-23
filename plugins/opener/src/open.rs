// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use std::path::Path;

#[cfg(desktop)]
pub(crate) fn open<P: AsRef<std::ffi::OsStr>, S: AsRef<str>>(
    path: P,
    with: Option<S>,
) -> crate::Result<()> {
    match with {
        Some(program) => ::open::with_detached(path, program.as_ref()),
        None => ::open::that_detached(path),
    }
    .map_err(Into::into)
}

/// Opens URL with the program specified in `with`, or system default if `None`.
///
/// This does not check the plugin's scope.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Unsupported, returns [`crate::Error::UnsupportedPlatform`].
///   Use [`crate::Opener::open_url`] instead.
///
/// # Examples
///
/// ```rust,no_run
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default browser
///     tauri_plugin_opener::open_url("https://github.com/tauri-apps/tauri", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_url<P: AsRef<str>, S: AsRef<str>>(url: P, with: Option<S>) -> crate::Result<()> {
    #[cfg(desktop)]
    {
        // `inAppBrowser` only has a meaning on mobile, open with the default browser instead
        let with = with
            .as_ref()
            .map(AsRef::as_ref)
            .filter(|with| *with != "inAppBrowser");
        open(url.as_ref(), with)
    }

    #[cfg(mobile)]
    {
        let _ = (url, with);
        Err(crate::Error::UnsupportedPlatform)
    }
}

/// Opens path with the program specified in `with`, or system default if `None`.
///
/// This does not check the plugin's scope. Returns an error if `with` is `None` and the path does
/// not exist.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Unsupported, returns [`crate::Error::UnsupportedPlatform`].
///   Use [`crate::Opener::open_path`] instead.
///
/// # Examples
///
/// ```rust,no_run
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given path with its default application
///     tauri_plugin_opener::open_path("/path/to/file", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_path<P: AsRef<Path>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    #[cfg(desktop)]
    {
        let path = path.as_ref();
        if with.is_none() {
            // Returns an IO error if not exists, and besides `exists()` is a shorthand for `metadata()`
            _ = path.metadata()?;
        }
        open(path, with)
    }

    #[cfg(mobile)]
    {
        let _ = (path, with);
        Err(crate::Error::UnsupportedPlatform)
    }
}
