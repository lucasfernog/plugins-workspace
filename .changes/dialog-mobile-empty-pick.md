---
dialog: patch
dialog-js: patch
---

Fixed a panic on Android and iOS when the native file picker resolves with an empty list of files; `pick_file` and the `open` command now return `None`/`null` instead.
