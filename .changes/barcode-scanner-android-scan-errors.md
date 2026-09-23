---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `scan()` never settling on Android when the device has no camera or the camera fails to start (for example when the requested front camera does not exist): it now rejects with the reason and removes the camera preview.
