---
autostart: patch
autostart-js: patch
---

Fixed the macOS Launch Agent being ignored by launchd when the app name, executable path or startup arguments contain XML special characters such as `&` or `<`. The values written to the property list are now escaped.
