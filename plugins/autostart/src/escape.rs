// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Helpers to write (escape) and read the platform's autostart entries.
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

/// Returns whether the `[Desktop Entry]` group of a desktop entry file is turned off with
/// `Hidden=true` or `X-GNOME-Autostart-enabled=false`, which desktop environments write when
/// the user disables an autostart entry in their settings.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(crate) fn desktop_entry_is_disabled(content: &str) -> bool {
    let mut in_desktop_entry = false;
    for line in content.lines().map(str::trim) {
        if line.starts_with('[') {
            in_desktop_entry = line == "[Desktop Entry]";
            continue;
        }
        if !in_desktop_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match (key.trim(), value.trim()) {
            ("Hidden", "true") | ("X-GNOME-Autostart-enabled", "false") => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_entry_is_disabled_reads_hidden_and_gnome_keys() {
        let entry = "[Desktop Entry]\nType=Application\nName=My App\nExec=/usr/bin/my-app\n";
        assert!(!desktop_entry_is_disabled(entry));
        assert!(desktop_entry_is_disabled(&format!("{entry}Hidden=true\n")));
        assert!(desktop_entry_is_disabled(&format!(
            "{entry}X-GNOME-Autostart-enabled = false\n"
        )));
        assert!(!desktop_entry_is_disabled(&format!(
            "{entry}Hidden=false\n"
        )));
        assert!(!desktop_entry_is_disabled(&format!(
            "{entry}X-GNOME-Autostart-enabled=true\n"
        )));
        // keys of other groups don't count
        assert!(!desktop_entry_is_disabled(&format!(
            "{entry}[Desktop Action x]\nHidden=true\n"
        )));
    }

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
