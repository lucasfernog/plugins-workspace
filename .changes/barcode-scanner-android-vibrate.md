---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed the `vibrate` command throwing a `NullPointerException` on Android when no scan had run yet, and doing nothing on Android 7 (API 24 and 25).
