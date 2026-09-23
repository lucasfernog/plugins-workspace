---
notification: patch
notification-js: patch
---

`sendNotification(options)` and `new Notification(title, options)` no longer mutate and freeze the options object passed by the caller.
