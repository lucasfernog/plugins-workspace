// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use serde::{Deserialize, Deserializer};
use url::Url;
use urlpattern::{UrlPattern, UrlPatternMatchInput};

/// A URL pattern of the opt-in `connect` scope.
#[derive(Debug)]
pub struct Entry {
    pub url: UrlPattern,
}

fn parse_url_pattern(s: &str) -> Result<UrlPattern, urlpattern::quirks::Error> {
    let mut init = urlpattern::UrlPatternInit::parse_constructor_string::<regex::Regex>(s, None)?;
    if init.search.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.search.replace("*".to_string());
    }
    if init.hash.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        init.hash.replace("*".to_string());
    }
    if init
        .pathname
        .as_ref()
        .map(|p| p.is_empty() || p == "/")
        .unwrap_or(true)
    {
        init.pathname.replace("*".to_string());
    }
    UrlPattern::parse(init, Default::default())
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum EntryRaw {
    Value(String),
    Object { url: String },
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        EntryRaw::deserialize(deserializer).and_then(|raw| {
            let url = match raw {
                EntryRaw::Value(url) => url,
                EntryRaw::Object { url } => url,
            };
            Ok(Entry {
                url: parse_url_pattern(&url).map_err(|e| {
                    serde::de::Error::custom(format!("`{url}` is not a valid URL pattern: {e}"))
                })?,
            })
        })
    }
}

/// The opt-in scope of the `connect` command.
///
/// Without any entry every URL is allowed, as before the scope existed. Denied entries always win;
/// once at least one allowed entry is configured, only matching URLs are allowed.
#[derive(Debug)]
pub struct Scope {
    allowed: Vec<Arc<Entry>>,
    denied: Vec<Arc<Entry>>,
}

impl Scope {
    pub(crate) fn new(allowed: Vec<Arc<Entry>>, denied: Vec<Arc<Entry>>) -> Self {
        Self { allowed, denied }
    }

    /// Whether the scope has any entry, i.e. whether it restricts anything at all.
    pub fn is_configured(&self) -> bool {
        !self.allowed.is_empty() || !self.denied.is_empty()
    }

    /// Determines if the given URL is allowed on this scope.
    pub fn is_allowed(&self, url: &Url) -> bool {
        let matches = |entry: &Arc<Entry>| {
            entry
                .url
                .test(UrlPatternMatchInput::Url(url.clone()))
                .unwrap_or_default()
        };
        if self.denied.iter().any(matches) {
            false
        } else {
            self.allowed.is_empty() || self.allowed.iter().any(matches)
        }
    }
}

#[cfg(test)]
impl std::str::FromStr for Entry {
    type Err = urlpattern::quirks::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = parse_url_pattern(s)?;
        Ok(Self { url })
    }
}

#[cfg(test)]
mod tests {
    use super::Scope;
    use std::sync::Arc;

    fn scope(allowed: &[&str], denied: &[&str]) -> Scope {
        let entries = |list: &[&str]| {
            list.iter()
                .map(|s| Arc::new(s.parse().unwrap()))
                .collect::<Vec<_>>()
        };
        Scope::new(entries(allowed), entries(denied))
    }

    fn allowed(scope: &Scope, url: &str) -> bool {
        scope.is_allowed(&url.parse().unwrap())
    }

    #[test]
    fn empty_scope_allows_everything() {
        let scope = scope(&[], &[]);
        assert!(!scope.is_configured());
        assert!(allowed(&scope, "ws://127.0.0.1:8080/socket"));
        assert!(allowed(&scope, "wss://example.com"));
    }

    #[test]
    fn allow_list_restricts() {
        let scope = scope(&["wss://example.com/*", "ws://localhost:*"], &[]);
        assert!(scope.is_configured());
        assert!(allowed(&scope, "wss://example.com/socket"));
        assert!(allowed(&scope, "wss://example.com/socket?token=1"));
        assert!(allowed(&scope, "ws://localhost:8080/"));
        assert!(!allowed(&scope, "ws://example.com/socket"));
        assert!(!allowed(&scope, "wss://evil.example.org/socket"));
        assert!(!allowed(&scope, "ws://127.0.0.1:8080/"));
    }

    #[test]
    fn deny_only_blocks_matches() {
        let scope = scope(&[], &["ws://127.0.0.1:*/admin/*"]);
        assert!(scope.is_configured());
        assert!(!allowed(&scope, "ws://127.0.0.1:9000/admin/socket"));
        assert!(allowed(&scope, "ws://127.0.0.1:9000/public"));
        assert!(allowed(&scope, "wss://example.com/admin/socket"));

        // the entry the examples/api capability uses for the e2e suite
        let scope = self::scope(&[], &["ws://127.0.0.1:*/ws/denied"]);
        assert!(!allowed(&scope, "ws://127.0.0.1:3004/ws/denied"));
        assert!(allowed(&scope, "ws://127.0.0.1:3004/ws"));
        assert!(allowed(&scope, "ws://127.0.0.1:3004/ws/headers"));
    }

    #[test]
    fn denied_takes_precedence() {
        let scope = scope(&["wss://example.com/*"], &["wss://example.com/private/*"]);
        assert!(allowed(&scope, "wss://example.com/public"));
        assert!(!allowed(&scope, "wss://example.com/private/socket"));
    }
}
