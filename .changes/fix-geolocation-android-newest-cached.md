---
geolocation: patch
geolocation-js: patch
---

Fixed `getCurrentPosition` on Android returning the oldest instead of the newest cached location when `maximumAge` is set.
