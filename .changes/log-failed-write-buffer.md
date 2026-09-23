---
log: patch
log-js: patch
---

Fixed a log record that failed to be written to a `Folder` or `LogDir` target (for example because rotating the file failed) being kept in memory and prepended to every following record while the error persisted. The failed record is now dropped.
