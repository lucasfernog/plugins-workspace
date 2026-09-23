---
barcode-scanner: patch
barcode-scanner-js: patch
---

Fixed an invisible camera view being left over the webview on iOS, blocking all touches, when `scan()` rejected because the camera permission was not granted.
