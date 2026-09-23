---
barcode-scanner: patch
barcode-scanner-js: patch
---

Implemented the `vibrate` command on iOS, which is registered and allowed by the default permission set but previously failed there because it had no iOS implementation.
