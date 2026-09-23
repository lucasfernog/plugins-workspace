---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `scan()` on iOS 13 always rejecting with "denied by the user" when the camera permission had not been granted yet, even when the user granted it in the prompt.
