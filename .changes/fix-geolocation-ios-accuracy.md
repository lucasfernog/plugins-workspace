---
geolocation: patch
geolocation-js: patch
---

Fixed a low-accuracy `getCurrentPosition` or `watchPosition` call on iOS lowering the accuracy of the high-accuracy watchers already running. The location manager now uses the highest accuracy any pending request or active watcher asked for.
