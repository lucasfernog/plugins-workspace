---
opener: patch
opener-js: patch
---

The injected link click handler now logs an error to the console when a link cannot be opened (e.g. because it is not allowed by the `openUrl` scope) instead of failing silently, and handles `Cmd`/`Meta`-clicks like `Ctrl`-clicks, so the macOS "open in new tab" gesture opens the link with the default browser.
