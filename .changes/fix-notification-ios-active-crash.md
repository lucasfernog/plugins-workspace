---
notification: patch
notification-js: patch
---

Fixed an iOS crash when a notification that was not shown by the running app process (e.g. scheduled before the app was restarted, or posted by another library) was tapped, presented in the foreground, or listed by `active()`.
