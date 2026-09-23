---
window-state: patch
window-state-js: patch
---

Restoring the state of a window excluded with `with_denylist` or `with_filter` (for example with `restoreState` from JavaScript) no longer starts tracking it, so its state is no longer saved to disk.
