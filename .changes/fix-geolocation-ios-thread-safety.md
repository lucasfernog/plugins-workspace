---
geolocation: patch
geolocation-js: patch
---

Fixed data races on iOS: the pending requests and watchers are now only changed on the main thread, where the location manager reports its updates.
