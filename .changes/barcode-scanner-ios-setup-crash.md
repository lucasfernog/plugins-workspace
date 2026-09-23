---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed a crash on iOS when the camera could not be opened (for example when it is in use or restricted): `scan()` now rejects with the reason and removes the camera view.
