// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Escaping helpers for the values written to the platform's autostart entries.
//!
//! They are pure functions so they can be unit tested on every platform.

/// Quotes `arg` so that Windows' command line parsing (`CommandLineToArgvW` and the C runtime)
/// reads it back as a single argument.
///
/// Arguments without whitespace or quotes are returned unchanged unless `always_quote` is set.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn windows_arg(arg: &str, always_quote: bool) -> String {
    if !always_quote && !arg.is_empty() && !arg.contains([' ', '\t', '\n', '\x0B', '"']) {
        return arg.to_string();
    }

    let mut quoted = String::with_capacity(arg.len() + 2);
    quoted.push('"');
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                // backslashes before a quote are escaped, and so is the quote itself
                quoted.extend(std::iter::repeat('\\').take(backslashes * 2 + 1));
                backslashes = 0;
            }
            _ => {
                quoted.extend(std::iter::repeat('\\').take(backslashes));
                backslashes = 0;
            }
        }
        if c != '\\' {
            quoted.push(c);
        }
    }
    // backslashes before the closing quote are escaped
    quoted.extend(std::iter::repeat('\\').take(backslashes * 2));
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_arg_keeps_plain_arguments() {
        assert_eq!(windows_arg("--autostarted", false), "--autostarted");
        assert_eq!(windows_arg(r"C:\dir\app.exe", false), r"C:\dir\app.exe");
    }

    #[test]
    fn windows_arg_quotes_whitespace_and_quotes() {
        assert_eq!(
            windows_arg(r"C:\Users\John Doe\AppData\Local\My App\my-app.exe", true),
            r#""C:\Users\John Doe\AppData\Local\My App\my-app.exe""#
        );
        assert_eq!(windows_arg(r"C:\app.exe", true), r#""C:\app.exe""#);
        assert_eq!(windows_arg("", false), "\"\"");
        assert_eq!(windows_arg("two words", false), r#""two words""#);
        assert_eq!(windows_arg(r#"a "b"#, false), r#""a \"b""#);
        assert_eq!(windows_arg(r#"a\"b"#, false), r#""a\\\"b""#);
        assert_eq!(windows_arg(r"C:\my dir\", false), r#""C:\my dir\\""#);
    }
}
