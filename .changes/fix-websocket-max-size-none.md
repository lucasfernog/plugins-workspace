---
websocket: patch
websocket-js: patch
---

Fixed `maxMessageSize` and `maxFrameSize` rejecting the documented `'none'` value in `WebSocket.connect`: `'none'` now removes the limit.
