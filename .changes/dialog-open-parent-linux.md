---
dialog: patch
dialog-js: patch
---

The `open` command now sets the calling window as the dialog's parent on Linux too, like `save` and `message` already did, so the file dialog is modal to and placed over the calling window.
