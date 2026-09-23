---
geolocation: patch
geolocation-js: patch
---

Fixed running several `watchPosition` watchers at once on Android: every watcher now keeps receiving updates, and `clearWatch` only stops the watcher it was given.
