---
log: patch
log-js: patch
---

Fixed a panic (debug builds) or unbounded log retention (release builds) when rotating a log file with `RotationStrategy::KeepSome(0)`. `KeepSome(0)` now keeps no archived file, like `RotationStrategy::KeepOne`.
