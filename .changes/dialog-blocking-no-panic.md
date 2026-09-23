---
dialog: patch
dialog-js: patch
---

The `blocking_*` dialog APIs, and the `open`, `save` and `message` commands that use them, no longer panic when the dialog callback is dropped without being called (for example when the dialog cannot be dispatched to the main thread because the app is exiting). They now return the same value as a cancelled dialog.
