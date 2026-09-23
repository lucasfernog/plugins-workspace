---
dialog: patch
dialog-js: patch
---

Fixed a data race on iOS when several photos or videos are picked at once, which could crash the app or drop files. The picked files are now also returned in the order they were selected.
