---
os: patch
os-js: patch
---

Marked the `allow-`/`deny-` permissions for `platform`, `version`, `os-type`, `family`, `arch` and `exe-extension` as deprecated. They never had an effect: those values are injected into every webview at startup, not read through commands. The identifiers are kept so existing capability files keep working, and will be removed in v3.
