---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `scan({ formats })` on Android always detecting QR codes as well, even when they were not in the requested `formats`.
