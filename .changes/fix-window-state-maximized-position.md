---
window-state: patch
window-state-js: patch
---

When restoring a maximized window, the position it is restored to (the one it had before it was maximized) is now the one checked against the available monitors, so a window whose pre-maximize position was on a disconnected monitor no longer ends up off-screen once it is unmaximized. The position is also set once instead of once per monitor the window overlaps.
