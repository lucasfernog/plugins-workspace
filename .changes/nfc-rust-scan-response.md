---
nfc: patch
nfc-js: patch
---

Fixed the Rust `Nfc::scan` API always failing to deserialize the scanned tag. `NfcTag::id` is now the tag identifier as a lowercase hexadecimal string, and `NfcTag::kind` is the list of technologies joined with `", "`.
