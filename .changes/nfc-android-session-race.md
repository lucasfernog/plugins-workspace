---
nfc: patch
nfc-js: patch
---

Fixed races on Android between a tag being processed and a new `scan` or `write` call, which could settle the wrong call or clear the new session.
