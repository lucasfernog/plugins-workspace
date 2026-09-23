---
log: patch
log-js: patch
---

Fixed archived log files being overwritten when a log file is rotated more than twice within the same second. Older archives of that second are now kept as `.log.bak`, `.log.bak.1`, `.log.bak.2`... and `RotationStrategy::KeepSome` now counts and prunes these `.bak` files, which previously accumulated forever.
