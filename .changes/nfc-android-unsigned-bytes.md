---
nfc: patch
nfc-js: patch
---

Fixed Android returning the tag `id` and the record `kind`, `id` and `payload` bytes as signed values (-128 to 127). Bytes of `0x80` and above are now reported as 128 to 255, like on iOS, which also fixes deserializing such records in the Rust `Nfc::scan` API.
