---
nfc: patch
nfc-js: patch
---

Fixed iOS dropping the record `id` of the records passed to `write`, including the `id` argument of `textRecord` and `uriRecord`.
