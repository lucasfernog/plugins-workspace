---
geolocation: patch
geolocation-js: patch
---

`altitude`, `altitudeAccuracy`, `speed` and `heading` are now `null` when the platform has no value for them, as documented, instead of `0` on Android or `-1` on iOS.
