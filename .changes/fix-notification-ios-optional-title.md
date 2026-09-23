---
notification: patch
notification-js: patch
---

Fixed notifications without a title (e.g. `app.notification().builder().body("...").show()` from Rust) failing on iOS.
