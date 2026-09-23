---
websocket: patch
websocket-js: patch
---

Fixed a race where `send` called right after `WebSocket.connect` resolved could fail with "connection not found": the connection is now registered before `connect` resolves.
