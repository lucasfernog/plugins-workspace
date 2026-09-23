---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed the camera preview staying on screen, and the camera starting anyway, on Android when `cancel()` was called before the camera finished initializing.
