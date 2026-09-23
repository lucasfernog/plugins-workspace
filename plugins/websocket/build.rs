// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[path = "src/scope.rs"]
#[allow(dead_code)]
mod scope;

const COMMANDS: &[&str] = &["connect", "send"];

/// WebSocket scope entry.
#[derive(schemars::JsonSchema)]
#[serde(untagged)]
#[allow(unused)]
enum WebSocketScopeEntry {
    /// A URL the webview can open a WebSocket connection to.
    /// Wildcards can be used following the URL pattern standard.
    ///
    /// See [the URL Pattern spec](https://urlpattern.spec.whatwg.org/) for more information.
    ///
    /// The scope is opt-in: without any entry every URL is allowed. Once an `allow` entry is
    /// configured only matching URLs are allowed, and `deny` entries always take precedence.
    ///
    /// Examples:
    ///
    /// - "wss://*" : allows all secure WebSocket origins on port 443
    ///
    /// - "wss://*:*" : allows all secure WebSocket origins on any port
    ///
    /// - "wss://api.example.com/socket/*": allows any URL that begins with "wss://api.example.com/socket/"
    Value(String),
    Object {
        /// A URL the webview can open a WebSocket connection to.
        /// Wildcards can be used following the URL pattern standard.
        ///
        /// See [the URL Pattern spec](https://urlpattern.spec.whatwg.org/) for more information.
        ///
        /// Examples:
        ///
        /// - "wss://*" : allows all secure WebSocket origins on port 443
        ///
        /// - "wss://*:*" : allows all secure WebSocket origins on any port
        ///
        /// - "wss://api.example.com/socket/*": allows any URL that begins with "wss://api.example.com/socket/"
        url: String,
    },
}

// Ensure `WebSocketScopeEntry` and `scope::EntryRaw` are kept in sync
fn _f() {
    match scope::EntryRaw::Value(String::new()) {
        scope::EntryRaw::Value(url) => WebSocketScopeEntry::Value(url),
        scope::EntryRaw::Object { url } => WebSocketScopeEntry::Object { url },
    };
    match WebSocketScopeEntry::Value(String::new()) {
        WebSocketScopeEntry::Value(url) => scope::EntryRaw::Value(url),
        WebSocketScopeEntry::Object { url } => scope::EntryRaw::Object { url },
    };
}

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .global_scope_schema(schemars::schema_for!(WebSocketScopeEntry))
        .build();
}
