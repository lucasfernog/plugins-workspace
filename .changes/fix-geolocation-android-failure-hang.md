---
geolocation: patch
geolocation-js: patch
---

Fixed `getCurrentPosition` hanging forever on Android when the location request failed with an exception that has no message. It now rejects.
