---
autostart: patch
autostart-js: patch
---

`enable()` on macOS now returns an error when the app runs from a temporary App Translocation path (a quarantined app launched from e.g. `~/Downloads`), instead of registering a path that no longer exists after a restart.
