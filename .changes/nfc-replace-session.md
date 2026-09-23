---
nfc: patch
nfc-js: patch
---

Fixed a pending `scan` or `write` call never settling when another `scan` or `write` call starts a new session. The previous call now rejects, and on iOS the previous NFC reader session is closed.
