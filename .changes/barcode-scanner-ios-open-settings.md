---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed `openAppSettings()` on iOS never settling when the settings page could not be opened; it now rejects in that case.
