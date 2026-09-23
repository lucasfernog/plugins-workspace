---
single-instance: patch
---

On macOS, the second instance now calls `AppHandle::cleanup_before_exit` before exiting, like it already did on Windows and Linux.
