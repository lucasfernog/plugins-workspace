---
autostart: patch
autostart-js: patch
---

Fixed `disable()` failing when autostart is already disabled on Windows and on macOS with `MacosLauncher::AppleScript`. It now succeeds, as it does with the macOS Launch Agent and on Linux.
