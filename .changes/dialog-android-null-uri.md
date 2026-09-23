---
dialog: patch
dialog-js: patch
---

On Android, a picked item without a URI is now skipped instead of making the whole file picker result fail to deserialize (which was reported as a cancelled picker).
