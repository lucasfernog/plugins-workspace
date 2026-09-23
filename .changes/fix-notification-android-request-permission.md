---
notification: patch
notification-js: patch
---

Fixed `requestPermission()` never resolving on Android 13+ when the notification permission was already granted, which also blocked the Rust `Notification::request_permission` call forever.
