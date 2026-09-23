---
window-state: patch
window-state-js: patch
---

Fixed `restoreStateCurrent` (and `restoreState` with the calling window's label) failing with "Couldn't find window" when called from a window that hosts several webviews.
