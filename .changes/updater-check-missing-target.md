---
updater: patch
updater-js: patch
---

`check()` now returns no update instead of failing with `TargetNotFound`/`TargetsNotFound` when the update manifest has no entry for the current target but the announced release is not an update anyway.
