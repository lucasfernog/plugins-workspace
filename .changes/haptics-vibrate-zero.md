---
"haptics": patch
"haptics-js": patch
---

`vibrate(0)` now stops the current vibration on Android and iOS, like the web Vibration API. It failed on Android, where a zero-length vibration is invalid, and did nothing on iOS.
