---
positioner: patch
positioner-js: patch
---

On Windows and macOS, `TrayLeft`, `TrayRight` and `TrayCenter` now decide whether to move the window below the tray icon by comparing against the top edge of the monitor the tray icon is on, instead of `y = 0`. This fixes the window ending up on another monitor or off-screen when the tray is on a monitor above or below the primary one.
