---
window-state: patch
window-state-js: patch
---

Fixed the check that decides whether a saved position is still on a monitor: a window larger than the monitor now counts as on it (only its corners were checked), and extreme values from a corrupt state file no longer overflow (a panic in debug builds).
