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

/// Escapes `value` for use as XML character data.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn xml_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            c => escaped.push(c),
        }
    }
    escaped
}

/// Returns the content of the Launch Agent property list that starts `program` with `args`
/// at login, with the `label` job label.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub(crate) fn launch_agent_plist(label: &str, program: &str, args: &[String]) -> String {
    let program_arguments = std::iter::once(program)
        .chain(args.iter().map(String::as_str))
        .map(|arg| format!("<string>{}</string>", xml_escape(arg)))
        .collect::<String>();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>{program_arguments}</array>
    <key>RunAtLoad</key>
    <true/>
  </dict>
</plist>"#,
        xml_escape(label)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_escapes_markup() {
        assert_eq!(xml_escape("Tom & Jerry"), "Tom &amp; Jerry");
        assert_eq!(
            xml_escape(r#"<a href="x">'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&apos;&lt;/a&gt;"
        );
        assert_eq!(xml_escape("plain"), "plain");
    }

    #[test]
    fn launch_agent_plist_escapes_label_and_arguments() {
        let plist = launch_agent_plist(
            "Tom & Jerry",
            "/Applications/Tom & Jerry.app/Contents/MacOS/Tom & Jerry",
            &["--from=<login>".into()],
        );
        assert!(plist.contains("<key>Label</key>\n    <string>Tom &amp; Jerry</string>"));
        assert!(plist.contains(
            "<array><string>/Applications/Tom &amp; Jerry.app/Contents/MacOS/Tom &amp; Jerry</string><string>--from=&lt;login&gt;</string></array>"
        ));
        assert!(!plist.contains("Tom & Jerry"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn launch_agent_plist_passes_plutil_lint() {
        let path = std::env::temp_dir().join(format!(
            "tauri-plugin-autostart-test-{}.plist",
            std::process::id()
        ));
        std::fs::write(
            &path,
            launch_agent_plist(
                "Tom & Jerry <\"'>",
                "/Applications/Tom & Jerry.app/Contents/MacOS/Tom & Jerry",
                &["--a=<b>".into(), "&".into()],
            ),
        )
        .unwrap();
        let status = std::process::Command::new("plutil")
            .arg("-lint")
            .arg(&path)
            .status()
            .unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(status.success());
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
