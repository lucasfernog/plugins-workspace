---
localhost: patch
---

Plugin setup now fails when the localhost server can't bind to its `host:port` (for example when the port is already in use). Previously the error only panicked a background thread and the app kept running without a server, so the window could load a page served by another local process on that port.
