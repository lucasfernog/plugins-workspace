---
nfc: patch
nfc-js: patch
---

Fixed `scan` and `write` never settling on iOS when connecting to, reading or writing the tag failed, or when the tag is read-only or not NDEF formatted. They now reject with the error.
