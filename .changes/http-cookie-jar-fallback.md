---
http: patch
http-js: patch
---

The plugin setup no longer fails, aborting the app startup, when the application cache directory or the `.cookies` file cannot be created or opened: the cookie jar falls back to an in-memory store.
