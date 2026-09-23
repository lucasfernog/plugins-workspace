---
updater: patch
updater-js: patch
---

On Windows, don't panic when installing an update with no current process arguments recorded (e.g. an `Updater` whose builder was not created through `UpdaterExt`).
