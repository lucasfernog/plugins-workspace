// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Escaping helpers for the values written to the platform's autostart entries.
//!
//! They are pure functions so they can be unit tested on every platform.

/// Returns `value` as a double-quoted AppleScript string literal.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn applescript_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for c in value.chars() {
        if matches!(c, '"' | '\\') {
            quoted.push('\\');
        }
        quoted.push(c);
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applescript_string_escapes_quotes_and_backslashes() {
        assert_eq!(applescript_string("My App"), r#""My App""#);
        assert_eq!(
            applescript_string(r#"a"b\c"#),
            r#""a\"b\\c""#,
            "quotes and backslashes must not end the literal"
        );
        assert_eq!(
            applescript_string(r#"x" & do shell script "id"#),
            r#""x\" & do shell script \"id""#
        );
    }
}
