---
fs: patch
fs-js: patch
---

**Security:** on Android and iOS, `file://` URLs passed to `open`, `readFile`, `readTextFile`, `writeFile`, `writeTextFile`, `stat` and `lstat` are now checked against the fs scope, like on desktop and like every other command. Previously they were opened without any scope check. Android `content://` URIs and `asset://localhost/` resources are still opened by the native layer without a scope check.
