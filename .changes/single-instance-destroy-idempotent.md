---
single-instance: patch
---

`destroy()` can now safely be called more than once, for example manually before the automatic call on `RunEvent::Exit`. On Windows it no longer closes the same handle twice, and on macOS it only removes the socket file this instance created, so it can't delete the socket of an instance started afterwards.
