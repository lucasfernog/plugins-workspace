// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

/// HTTP plugin configuration, defined on the `plugins > http` object of your `tauri.conf.json`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Whether the scope is checked on every hop of a redirect chain instead of only on the
    /// URL requested by the frontend. Defaults to `false`.
    ///
    /// When disabled, a server on an allowed origin can answer with a redirect to any other
    /// origin - an open redirect, or a server an attacker controls - and the plugin follows it,
    /// handing the webview a response from a URL the scope denies, such as a `localhost`
    /// service, an internal host or a cloud metadata endpoint.
    ///
    /// When enabled, a redirect to a URL that is not allowed by the scope fails with
    /// [`Error::UrlNotAllowed`](crate::Error::UrlNotAllowed) instead of being followed, so every
    /// redirect target must also be part of the scope. This is opt-in because it breaks
    /// applications that rely on being redirected outside of their configured scope.
    // TODO(v3): enforce the scope on redirects by default and remove this option
    #[serde(default)]
    pub scope_redirects: bool,
    /// Whether the proxy URLs the frontend sets with the `proxy` option of `fetch` must be
    /// allowed by the scope. Defaults to `false`.
    ///
    /// When disabled, the frontend can route an allowed request through any `host:port`, which
    /// makes the plugin open connections to hosts the scope does not allow - and, for `http://`
    /// URLs, hand the webview the response of that host.
    ///
    /// When enabled, a proxy URL that is not allowed by the scope makes the request fail with
    /// [`Error::UrlNotAllowed`](crate::Error::UrlNotAllowed), so every proxy the frontend uses
    /// (for instance `http://proxy.example.com:8080`) must be added to the scope. This is opt-in
    /// because it breaks applications that let the frontend pick a proxy outside of their scope.
    // TODO(v3): enforce the scope on proxies by default and remove this option
    #[serde(default)]
    pub scope_proxy: bool,
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn deserializes_the_plugin_configuration() {
        // the plugin configuration is optional, and the scope check is opt-in
        let config: Option<Config> = serde_json::from_value(serde_json::Value::Null).unwrap();
        assert!(config.is_none());

        let config: Config = serde_json::from_str("{}").unwrap();
        assert!(!config.scope_redirects);
        assert!(!config.scope_proxy);

        let config: Config = serde_json::from_str(r#"{ "scopeProxy": true }"#).unwrap();
        assert!(config.scope_proxy);

        let config: Config = serde_json::from_str(r#"{ "scopeRedirects": true }"#).unwrap();
        assert!(config.scope_redirects);

        // a typo must not silently disable the scope check
        assert!(serde_json::from_str::<Config>(r#"{ "scopeRedirect": true }"#).is_err());
    }
}
