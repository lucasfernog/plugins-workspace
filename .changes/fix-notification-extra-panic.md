---
notification: patch
notification-js: patch
---

`NotificationBuilder::extra` no longer panics when the value cannot be serialized to JSON (e.g. a map with non-string keys). The value is skipped and an error is logged.
