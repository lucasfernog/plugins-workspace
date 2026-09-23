---
websocket: patch
websocket-js: patch
---

Fixed connections staying registered after the stream ended without a Close frame (network drop, reset or protocol error). `send` on such a connection now fails with "connection not found" instead of a stream error.
