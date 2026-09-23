---
opener: patch
opener-js: patch
---

Fixed the `openPath` scope check accepting a path that does not exist and uses `..` components to leave an allowed directory, e.g. `$APPDATA/missing/../../file` matching `$APPDATA/**` on Windows or when `requireLiteralLeadingDot` is `false`. Such paths are now rejected.
