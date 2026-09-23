---
log: patch
log-js: patch
---

Fixed `RotationStrategy::KeepSome` deleting files that belong to other log targets whose file name starts with the same prefix (for example the active `app_webview.log` when pruning the archives of the `app` target). Only files named `{file_name}_{date}.log` are now treated as archives.
