---
deep-link: patch
deep-link-js: patch
---

On Android, no longer emit a `deep-link://new-url` event with a `[null]` payload when the URL sent by the native side cannot be read.
