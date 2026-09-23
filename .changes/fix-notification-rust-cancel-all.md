---
notification: patch
notification-js: patch
---

Fixed the Rust `Notification::cancel_all` and `Notification::remove_all_active` methods failing on mobile. They sent `null` arguments, which the Android plugin could not parse (and the iOS plugin could not parse for `cancel_all`).
