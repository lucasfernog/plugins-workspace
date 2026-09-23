---
nfc: patch
nfc-js: patch
---

Fixed `textRecord` setting the language code length from its string length instead of its UTF-8 byte length. The status byte now also keeps the UTF-8 bit and the reserved bit cleared.
