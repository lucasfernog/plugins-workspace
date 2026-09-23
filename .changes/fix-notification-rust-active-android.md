---
notification: patch
notification-js: patch
---

Fixed the Rust `Notification::active` method failing on Android. The notification extras that are not strings are reported as `null`, which could not be deserialized into `ActiveNotification::data`; they are now skipped.
