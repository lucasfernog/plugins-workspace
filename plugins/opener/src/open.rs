// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use std::{ffi::OsStr, path::Path};

pub(crate) fn open<P: AsRef<OsStr>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    match with {
        #[cfg(windows)]
        Some(program) => windows_impl::with_detached(path.as_ref(), program.as_ref()),
        #[cfg(not(windows))]
        Some(program) => ::open::with_detached(path, program.as_ref()).map_err(Into::into),
        None => ::open::that_detached(path).map_err(Into::into),
    }
}

/// Opening with a specific program on Windows.
///
/// `open::with_detached` passes the path or URL to `ShellExecuteExW` as the raw `lpParameters`
/// command line, so a path with spaces was split into several arguments, and a URL with spaces
/// could inject extra arguments into the program's command line. This quotes it as a single
/// argument instead.
#[cfg(windows)]
mod windows_impl {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

    use windows::{
        core::{HSTRING, PCWSTR},
        Win32::UI::{
            Shell::{ShellExecuteExW, SHELLEXECUTEINFOW},
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
    };

    pub fn with_detached(path: &OsStr, program: &str) -> crate::Result<()> {
        let program = HSTRING::from(program);
        let path: Vec<u16> = path.encode_wide().collect();
        let parameters = HSTRING::from_wide(&super::quote_windows_arg(&path));

        let mut info = SHELLEXECUTEINFOW {
            cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as _,
            nShow: SW_SHOWNORMAL.0,
            lpFile: PCWSTR(program.as_ptr()),
            lpParameters: PCWSTR(parameters.as_ptr()),
            ..Default::default()
        };

        unsafe { ShellExecuteExW(&mut info) }.map_err(Into::into)
    }
}

/// Quotes `arg` (UTF-16) as a single argument of a Windows command line, following the
/// `CommandLineToArgvW` / MSVC CRT parsing rules.
#[cfg(any(windows, test))]
fn quote_windows_arg(arg: &[u16]) -> Vec<u16> {
    const QUOTE: u16 = b'"' as u16;
    const BACKSLASH: u16 = b'\\' as u16;

    let mut quoted = Vec::with_capacity(arg.len() + 2);
    quoted.push(QUOTE);
    let mut backslashes = 0;
    for &c in arg {
        if c == BACKSLASH {
            backslashes += 1;
        } else {
            if c == QUOTE {
                // escape the preceding backslashes and the quote itself
                quoted.resize(quoted.len() + backslashes + 1, BACKSLASH);
            }
            backslashes = 0;
        }
        quoted.push(c);
    }
    // escape trailing backslashes so they do not escape the closing quote
    quoted.resize(quoted.len() + backslashes, BACKSLASH);
    quoted.push(QUOTE);
    quoted
}

/// Opens URL with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
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
    let url = url.as_ref();
    open(url, with)
}

/// Opens path with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
///
/// # Examples
///
/// ```rust,no_run
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default explorer
///     tauri_plugin_opener::open_path("/path/to/file", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_path<P: AsRef<Path>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    let path = path.as_ref();
    if with.is_none() {
        // Returns an IO error if not exists, and besides `exists()` is a shorthand for `metadata()`
        _ = path.metadata()?;
    }
    open(path, with)
}

#[cfg(test)]
mod tests {
    use super::quote_windows_arg;

    fn quote(arg: &str) -> String {
        let arg: Vec<u16> = arg.encode_utf16().collect();
        String::from_utf16(&quote_windows_arg(&arg)).unwrap()
    }

    #[test]
    fn quotes_windows_arguments() {
        assert_eq!(quote(r"C:\My Files\a.txt"), r#""C:\My Files\a.txt""#);
        assert_eq!(quote(""), r#""""#);
        assert_eq!(
            quote(r#"https://a.com --gpu-launcher="cmd /c calc""#),
            r#""https://a.com --gpu-launcher=\"cmd /c calc\"""#
        );
        assert_eq!(quote(r"C:\dir\"), r#""C:\dir\\""#);
        assert_eq!(quote(r#"a\"b"#), r#""a\\\"b""#);
        assert_eq!(quote(r"a\\b"), r#""a\\b""#);
    }
}
