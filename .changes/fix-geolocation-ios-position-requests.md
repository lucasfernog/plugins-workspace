---
geolocation: patch
geolocation-js: patch
---

Fixed `getCurrentPosition` calls on iOS never being removed from the pending list after they resolved, which resolved them again on every later location update, triggered extra location requests and leaked memory.
