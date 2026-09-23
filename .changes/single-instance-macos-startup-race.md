---
single-instance: patch
---

On macOS, two instances started at the same time can no longer both become the first instance. The first instance now binds its socket synchronously while holding an exclusive lock on a `<socket>.lock` file next to it (released automatically when the process exits), and an instance that finds the lock taken waits up to 5 seconds for the first instance to start listening and notifies it.
