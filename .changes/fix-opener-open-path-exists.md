---
opener: patch
opener-js: patch
---

Fixed the `openPath` command and `Opener::open_path` on desktop resolving successfully for a path that does not exist when no `with` program is given. They now return the I/O error, like the `open_path` function already did.
