---
nfc: patch
nfc-js: patch
---

Fixed `scan` and `write` never settling on iOS when the detected tag type or its NDEF status is not supported. They now reject.
