---
http: minor
http-js: minor
---

Added the `scopeProxy` configuration option (`plugins > http > scopeProxy` in `tauri.conf.json`). When enabled, the proxy URLs set by the frontend with the `proxy` option of `fetch` must be allowed by the scope, like the requested URL; otherwise the request fails with a "url not allowed on the configured scope" error. It is disabled by default, in which case a proxy lets the frontend reach any host and port.
