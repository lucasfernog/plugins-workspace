---
window-state: patch
window-state-js: patch
---

Restoring a window's state no longer drops the move and resize events of every other window, and no longer blocks another window's restore (which could deadlock when one ran on the main thread and the other in a `restoreState` call from JavaScript). The "restore in progress" state is now tracked per window.
