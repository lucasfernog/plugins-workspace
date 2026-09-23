---
websocket: patch
websocket-js: patch
---

Fixed repeated header names passed to the `connect` command replacing each other: the first value still replaces the request's default, and further values are now appended. (The JS API already merges repeated headers through `Headers`, so this only affects direct `invoke` calls.)
