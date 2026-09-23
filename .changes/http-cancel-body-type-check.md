---
http: patch
http-js: patch
---

`fetch_cancel_body` now only closes HTTP response bodies. Previously it closed any resource of the webview's resource table, including resources owned by other plugins.
