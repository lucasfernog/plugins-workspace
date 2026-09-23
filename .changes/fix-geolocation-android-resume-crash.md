---
geolocation: patch
geolocation-js: patch
---

Fixed a possible crash on Android when the app returned to the foreground with active watchers after the location permission was revoked. The watchers now receive an error instead.
