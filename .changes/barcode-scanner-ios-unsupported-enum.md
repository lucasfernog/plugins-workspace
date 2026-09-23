---
barcode-scanner: patch
barcode-scanner-js: patch
---

`scan()` on iOS now rejects `Format.UPC_A` and `Format.Codabar` with an "Unsupported barcode format" error instead of a generic argument decoding error.
