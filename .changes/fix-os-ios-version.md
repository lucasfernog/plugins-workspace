---
os: patch
os-js: patch
---

Fixed `version()` returning `Unknown` on iOS by requiring `os_info` 3.14, the first release that detects the iOS version.
