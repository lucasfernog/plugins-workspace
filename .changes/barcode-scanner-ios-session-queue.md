---
barcode-scanner: patch
barcode-scanner-js: patch
---

The iOS capture session is now started and stopped on a background queue instead of the main thread, which fixes the Thread Performance Checker warning and the UI stall when a scan starts.
