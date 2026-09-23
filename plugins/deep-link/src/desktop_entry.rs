// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Minimal, line based editing of freedesktop key files (`.desktop` entries and
//! `mimeapps.list`).
//!
//! A generic INI parser is not a good fit for these files: it strips the quotes of
//! `Exec="/path with spaces/app" %u`, applies its own escaping and drops comments when
//! writing the file back. Editing line by line leaves everything we don't touch as is.

/// Returns the group name if `line` is a group header (`[Group Name]`).
fn group_header(line: &str) -> Option<&str> {
    let line = line.trim();
    line.strip_prefix('[')?.strip_suffix(']')
}

/// Returns the key and value of a `Key=Value` line, `None` for comments, blank lines and
/// group headers.
fn key_value(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let (key, value) = trimmed.split_once('=')?;
    Some((key.trim(), value.trim()))
}

/// Splits `content` into lines, and the range of line indices (header excluded) of the first
/// `group`.
fn group_range<'a>(content: &'a str, group: &str) -> (Vec<&'a str>, Option<(usize, usize)>) {
    let lines = content.lines().collect::<Vec<_>>();
    let Some(header) = lines.iter().position(|l| group_header(l) == Some(group)) else {
        return (lines, None);
    };
    let end = lines[header + 1..]
        .iter()
        .position(|l| group_header(l).is_some())
        .map_or(lines.len(), |i| header + 1 + i);
    (lines, Some((header + 1, end)))
}

fn join(lines: &[impl AsRef<str>], original: &str) -> String {
    let mut out = lines
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>()
        .join("\n");
    if original.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Whether `content` has the given group.
pub fn has_group(content: &str, group: &str) -> bool {
    content.lines().any(|l| group_header(l) == Some(group))
}

/// Returns the value of `key` in `group`.
pub fn get(content: &str, group: &str, key: &str) -> Option<String> {
    let (lines, range) = group_range(content, group);
    let (start, end) = range?;
    lines[start..end]
        .iter()
        .filter_map(|l| key_value(l))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
}

/// Sets (or with `None`, removes) `key` in `group`, keeping every other line as is.
///
/// Returns `None` if the group does not exist.
pub fn set(content: &str, group: &str, key: &str, value: Option<&str>) -> Option<String> {
    let (lines, range) = group_range(content, group);
    let (start, end) = range?;

    let mut out = Vec::with_capacity(lines.len() + 1);
    out.extend(lines[..start].iter().map(|l| l.to_string()));

    let mut written = false;
    for line in &lines[start..end] {
        if key_value(line).is_some_and(|(k, _)| k == key) {
            // replace the first occurrence, drop duplicates
            if let (Some(value), false) = (value, written) {
                out.push(format!("{key}={value}"));
            }
            written = true;
        } else {
            out.push(line.to_string());
        }
    }

    if let (Some(value), false) = (value, written) {
        // append after the last non blank line of the group
        let insert_at = (start..out.len())
            .rev()
            .find(|i| !out[*i].trim().is_empty())
            .map_or(start, |i| i + 1);
        out.insert(insert_at, format!("{key}={value}"));
    }

    out.extend(lines[end..].iter().map(|l| l.to_string()));
    Some(join(&out, content))
}

/// Whether the output of `xdg-mime query default <mime type>` lists `desktop_file`.
pub fn is_default_handler(xdg_mime_output: &str, desktop_file: &str) -> bool {
    xdg_mime_output
        .split(|c: char| c == ';' || c.is_whitespace())
        .any(|h| h == desktop_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DESKTOP: &str = "# generated\n[Desktop Entry]\nType=Application\nName=App\nExec=\"/home/u/My Apps/app\" %u\nMimeType=x-scheme-handler/a;x-scheme-handler/b;\nNoDisplay=true\n\n[Desktop Action new]\nName=New\nExec=app --new\n";

    #[test]
    fn get_reads_the_right_group() {
        assert_eq!(
            get(DESKTOP, "Desktop Entry", "Exec").as_deref(),
            Some("\"/home/u/My Apps/app\" %u")
        );
        assert_eq!(
            get(DESKTOP, "Desktop Action new", "Exec").as_deref(),
            Some("app --new")
        );
        assert_eq!(get(DESKTOP, "Desktop Entry", "Missing"), None);
        assert_eq!(get(DESKTOP, "Missing", "Exec"), None);
        assert!(has_group(DESKTOP, "Desktop Entry"));
        assert!(!has_group(DESKTOP, "Missing"));
    }

    #[test]
    fn set_keeps_quotes_comments_and_other_groups() {
        let out = set(
            DESKTOP,
            "Desktop Entry",
            "MimeType",
            Some("x-scheme-handler/b;"),
        )
        .unwrap();
        assert_eq!(
            out,
            DESKTOP.replace(
                "MimeType=x-scheme-handler/a;x-scheme-handler/b;",
                "MimeType=x-scheme-handler/b;"
            )
        );
    }

    #[test]
    fn set_removes_and_appends() {
        let removed = set(DESKTOP, "Desktop Entry", "MimeType", None).unwrap();
        assert_eq!(
            removed,
            DESKTOP.replace("MimeType=x-scheme-handler/a;x-scheme-handler/b;\n", "")
        );

        let appended = set(&removed, "Desktop Entry", "MimeType", Some("x/y;")).unwrap();
        assert_eq!(
            appended,
            DESKTOP.replace(
                "MimeType=x-scheme-handler/a;x-scheme-handler/b;\nNoDisplay=true\n",
                "NoDisplay=true\nMimeType=x/y;\n"
            )
        );

        assert_eq!(set(DESKTOP, "Missing", "MimeType", Some("x")), None);
    }

    #[test]
    fn default_handler_is_matched_exactly() {
        assert!(is_default_handler(
            "app-handler.desktop\n",
            "app-handler.desktop"
        ));
        assert!(is_default_handler(
            "other.desktop;app-handler.desktop;\n",
            "app-handler.desktop"
        ));
        assert!(!is_default_handler(
            "myapp-handler.desktop\n",
            "app-handler.desktop"
        ));
        assert!(!is_default_handler("", "app-handler.desktop"));
    }
}
