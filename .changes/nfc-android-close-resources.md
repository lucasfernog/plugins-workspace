---
nfc: patch
nfc-js: patch
---

Fixed the Android tag connection staying open when a write fails, and the NFC foreground dispatch staying enabled after a `scan` without `keepSessionAlive` resolved.
