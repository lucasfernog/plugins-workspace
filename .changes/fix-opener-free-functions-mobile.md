---
opener: patch
opener-js: patch
---

The `open_url` and `open_path` functions now return `Error::UnsupportedPlatform` on Android and iOS, where they previously tried to spawn desktop programs that do not exist there (use `app.opener().open_url()` / `open_path()` instead). On desktop, `open_url` now ignores `with: "inAppBrowser"` like `Opener::open_url` does, instead of trying to launch a program with that name.
