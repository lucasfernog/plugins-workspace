---
nfc: patch
nfc-js: patch
---

Fixed `write` never settling on iOS when it writes to the tag of a `scan` session kept alive with `keepSessionAlive`.
