---
geolocation: patch
geolocation-js: patch
---

The JavaScript commands no longer block an async runtime worker thread while a location or permission request is pending on mobile, which could starve the app's other async commands.
