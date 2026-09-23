---
shell: patch
shell-js: patch
---

Fixed the deprecated JavaScript `open()` (and the `<a target="_blank">` links it handles) always failing on Android and iOS: the command used the desktop implementation, which has no way to open anything there. It now goes through the native plugin, after the same `plugins > shell > open` validation. The `openWith` argument is ignored on mobile.
