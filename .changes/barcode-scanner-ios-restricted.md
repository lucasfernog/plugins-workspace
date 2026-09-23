---
barcode-scanner: patch
barcode-scanner-js: patch
---

`checkPermissions()` on iOS now reports `denied` instead of `prompt` when camera access is restricted (parental controls or device management), since requesting it can never succeed.
