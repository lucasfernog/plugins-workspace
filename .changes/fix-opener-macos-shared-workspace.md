---
opener: patch
opener-js: patch
---

`revealItemInDir` on macOS now uses the shared `NSWorkspace` instance instead of allocating a new one.
