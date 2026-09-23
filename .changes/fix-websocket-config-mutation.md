---
websocket: patch
websocket-js: patch
---

Fixed `WebSocket.connect` replacing `config.headers` on the caller's object with an array of entries; the headers are now converted on a copy.
