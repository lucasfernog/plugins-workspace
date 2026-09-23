---
geolocation: patch
geolocation-js: patch
---

`getCurrentPosition` and `watchPosition` now accept partial options, like the W3C Geolocation API: a missing `enableHighAccuracy` defaults to `false`, `timeout` to `10000` and `maximumAge` to `0`. `Infinity` (serialized as `null`) is accepted too, and out-of-range numbers are clamped instead of failing with a deserialization error.
