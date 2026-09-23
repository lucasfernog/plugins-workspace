---
updater: patch
updater-js: patch
---

The `install` and `downloadAndInstall` commands now run the install step on a blocking thread, so extracting the update, waiting for a password prompt or running the package manager no longer stalls the async runtime.
