---
window-state: patch
window-state-js: patch
---

The state file is now written to a temporary file and renamed into place, so a crash or kill while saving no longer leaves a truncated file behind (which discarded all saved state on the next launch).
