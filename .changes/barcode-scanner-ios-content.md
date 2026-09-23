---
barcode-scanner: patch
barcode-scanner-js: patch
---

`scan()` on iOS now always returns a `content` string (empty when the barcode has no string representation, such as a binary QR code), matching Android and the `Scanned` type.
