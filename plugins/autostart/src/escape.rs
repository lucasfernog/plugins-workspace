// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Escaping helpers for the values written to the platform's autostart entries.
//!
//! They are pure functions so they can be unit tested on every platform.

/// Quotes `arg` as one argument of a desktop entry's `Exec` key, following the
/// [Desktop Entry specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/exec-variables.html),
/// and escapes it as a desktop entry string value.
///
/// Arguments without reserved characters are returned unchanged.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn desktop_entry_exec_arg(arg: &str) -> String {
    const RESERVED: &[char] = &[
        ' ', '\t', '\n', '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(',
        ')', '`',
    ];

    let mut exec_arg = String::with_capacity(arg.len() + 2);
    if arg.is_empty() || arg.contains(RESERVED) {
        exec_arg.push('"');
        for c in arg.chars() {
            if matches!(c, '"' | '`' | '$' | '\\') {
                exec_arg.push('\\');
            }
            exec_arg.push(c);
        }
        exec_arg.push('"');
    } else {
        exec_arg.push_str(arg);
    }

    // `%` introduces field codes, and the key's value is a string with its own escapes.
    let mut escaped = String::with_capacity(exec_arg.len());
    for c in exec_arg.chars() {
        match c {
            '%' => escaped.push_str("%%"),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_exec_arg_keeps_plain_arguments() {
        assert_eq!(desktop_entry_exec_arg("/usr/bin/my-app"), "/usr/bin/my-app");
        assert_eq!(desktop_entry_exec_arg("--autostarted"), "--autostarted");
        assert_eq!(desktop_entry_exec_arg("--mode=a,b"), "--mode=a,b");
    }

    #[test]
    fn desktop_entry_exec_arg_quotes_reserved_characters() {
        assert_eq!(
            desktop_entry_exec_arg("/home/me/Apps/My App.AppImage"),
            r#""/home/me/Apps/My App.AppImage""#
        );
        assert_eq!(desktop_entry_exec_arg(""), "\"\"");
        assert_eq!(desktop_entry_exec_arg("a&b"), r#""a&b""#);
        // inside quotes `"`, `` ` ``, `$` and `\` are escaped with a backslash, then every
        // backslash is escaped again for the string value
        assert_eq!(desktop_entry_exec_arg(r#"a "b"#), r#""a \\"b""#);
        assert_eq!(desktop_entry_exec_arg("$HOME"), r#""\\$HOME""#);
        assert_eq!(desktop_entry_exec_arg(r"C:\x"), r#""C:\\\\x""#);
        assert_eq!(desktop_entry_exec_arg("a\nb"), r#""a\nb""#);
    }

    #[test]
    fn desktop_entry_exec_arg_escapes_field_codes() {
        assert_eq!(desktop_entry_exec_arg("100%"), "100%%");
        assert_eq!(desktop_entry_exec_arg("--file=%u"), "--file=%%u");
    }
}
