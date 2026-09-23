---
fs: patch
fs-js: patch
---

**Security:** deny patterns of the fs scope now also apply to the content of the directories that recursive operations work on. `remove` with `recursive: true` and `rename` of a directory are rejected when an entry below the directory (or its new location) is denied, `readDir` no longer lists denied entries, `size` skips them, and `watch` does not report changes of denied paths. Previously only the given path was checked, so e.g. with the default `$APPLOCALDATA/EBWebView/**` deny on Windows, `remove('EBWebView', { recursive: true })` succeeded.
