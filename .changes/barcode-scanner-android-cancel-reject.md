---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `cancel()` on Android never rejecting the pending `scan()` promise: it now rejects with `cancelled`, as it does on iOS.
