---
autostart: patch
autostart-js: patch
---

Fixed enabling autostart with `MacosLauncher::AppleScript` when the app path contains a `"` or `\`. The name and path are now escaped before they are passed to AppleScript.
