---
notification: patch
notification-js: patch
---

`isPermissionGranted()` now always resolves to a boolean (it resolved to `null` on mobile while the user had not decided yet), and `requestPermission()` / `window.Notification.requestPermission()` now resolve to a Notification Web API value (`'default'` instead of `'prompt'` or `'prompt-with-rationale'`), as their types declare.
