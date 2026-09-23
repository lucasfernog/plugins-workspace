---
autostart: patch
autostart-js: patch
---

`disable()` on Windows now also removes the `StartupApproved\Run` (Task Manager startup apps) value that `enable()` writes, instead of leaving a stale registry entry behind.
