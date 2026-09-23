---
single-instance: patch
---

On Windows, an instance started while the first instance is still starting up or shutting down no longer keeps running as a second primary instance. It now waits up to 5 seconds for the first instance's message window to appear, or for the first instance to exit and release its mutex (in which case it becomes the first instance). The first instance also destroys its message window before releasing the mutex on exit, and a mutex handle leak on this path was fixed.
