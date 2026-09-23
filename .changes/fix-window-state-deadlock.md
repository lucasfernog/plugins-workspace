---
window-state: patch
window-state-js: patch
---

Fixed a deadlock (the app freezing) when `saveWindowState`/`restoreState` were called from JavaScript while a window was being moved, resized or closed: the window state cache is no longer locked while querying the window.
