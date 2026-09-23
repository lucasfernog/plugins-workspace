---
nfc: patch
nfc-js: patch
---

Fixed Android enabling the NFC foreground dispatch from the command thread instead of the main thread when `scan` or `write` is called.
