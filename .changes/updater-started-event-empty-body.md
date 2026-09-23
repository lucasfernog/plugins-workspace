---
updater: patch
updater-js: patch
---

The download progress callback now always receives a `Started` event before `Finished`, also when the download body is empty.
