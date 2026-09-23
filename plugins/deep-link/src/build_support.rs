// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This module is imported in build.rs, and in the crate only for its unit tests.

//! Helpers for the files `build.rs` generates or edits from the `mobile` configuration.

use crate::config::AssociatedDomain;

/// Escapes a value for an XML attribute delimited by `"`.
pub fn xml_escape(value: &str) -> String {
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

/// The custom (non http/https) schemes of a domain, the ones iOS registers in
/// `CFBundleURLTypes`.
#[allow(dead_code)] // only used on macOS hosts
pub fn custom_schemes(domain: &AssociatedDomain) -> Vec<&str> {
    domain
        .scheme
        .iter()
        .map(String::as_str)
        .filter(|scheme| *scheme != "https" && *scheme != "http")
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain(json: &str) -> AssociatedDomain {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn escapes_xml_attributes() {
        assert_eq!(xml_escape("/a.*b"), "/a.*b");
        assert_eq!(
            xml_escape(r#"/a?b=1&c="<d>"'"#),
            "/a?b=1&amp;c=&quot;&lt;d&gt;&quot;&apos;"
        );
    }

    #[test]
    fn custom_schemes_skip_web_schemes() {
        assert_eq!(
            custom_schemes(&domain(r#"{ "scheme": ["https", "myapp"] }"#)),
            ["myapp"]
        );
        assert!(custom_schemes(&domain(r#"{ "scheme": [] }"#)).is_empty());
        assert!(custom_schemes(&domain(r#"{ "host": "a.b" }"#)).is_empty());
    }
}
