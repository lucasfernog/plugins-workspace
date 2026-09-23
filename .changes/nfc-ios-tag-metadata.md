---
nfc: patch
nfc-js: patch
---

On iOS, tags scanned with the `ndef` kind now always include `id` and `kind`. FeliCa tags report an empty `id` instead of none, and unrecognized tags report `["Unknown"]`, like the `tag` kind does.
