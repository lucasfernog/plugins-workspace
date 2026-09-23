---
global-shortcut: patch
global-shortcut-js: patch
---

Fixed a deadlock when a Rust shortcut handler calls back into the plugin, for example to unregister the shortcut or check `is_registered`. Handlers now run without the plugin's internal lock held, and on Linux they run on the main thread like on Windows and macOS instead of on global-hotkey's X11 event thread.
