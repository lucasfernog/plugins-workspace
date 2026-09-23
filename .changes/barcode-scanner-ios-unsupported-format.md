---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `scan()` on iOS still opening the camera after rejecting a format that is unsupported on the running iOS version (the GS1 DataBar formats before iOS 15.4).
