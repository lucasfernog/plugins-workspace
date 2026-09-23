---
geolocation: patch
geolocation-js: patch
---

Fixed `watchPosition` never resolving on Android: the promise (and the Rust `watch_position` call) now returns the watcher id once the watch is registered, instead of hanging forever and blocking an async runtime thread per call.
