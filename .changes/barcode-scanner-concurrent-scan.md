---
barcode-scanner: patch
barcode-scanner-js: patch
---

Starting a new `scan()` while another one is running now rejects the previous scan with `cancelled` instead of leaving its promise pending forever. On iOS this also no longer leaves the previous camera view on screen.
