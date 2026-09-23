---
websocket: patch
websocket-js: patch
---

Fixed a slow or backpressured connection blocking `send` on every other connection: each connection now has its own write lock instead of one lock held across network I/O for all of them.
