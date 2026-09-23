---
upload: patch
upload-js: patch
---

Fixed the `transferSpeed` reported in upload and download progress events. It is now measured in bytes per second over the real elapsed time, instead of a truncated, `* 1024` value that stayed at `0` on fast transfers (chunks less than 1 ms apart) and on slow ones (below 1 byte/ms).
