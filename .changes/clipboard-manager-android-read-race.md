---
clipboard-manager: patch
clipboard-manager-js: patch
---

Android: `readText` no longer fails with a `NullPointerException` when the clipboard changes while it is being read, and on Android 10+ its "Clipboard is empty" error now mentions that the clipboard can only be read while the app has input focus.
