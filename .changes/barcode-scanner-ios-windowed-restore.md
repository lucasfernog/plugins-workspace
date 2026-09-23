---
barcode-scanner: patch
barcode-scanner-js: patch
---

On iOS, a windowed scan now restores the webview's previous `isOpaque` value instead of always making it opaque, and an unknown `cameraDirection` now selects the back camera, as on Android.
