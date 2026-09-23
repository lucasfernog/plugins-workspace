---
single-instance: patch
---

On macOS, the socket used to talk to the first instance now lives in the per-user temporary directory (`$TMPDIR`, inside the app container for sandboxed apps) instead of the world-shared `/tmp`, so other users can no longer squat it, receive the app's arguments, or disable single-instance protection for each other, and sandboxed apps can use it. Both ends also check that the other process runs as the same user. For compatibility with instances of the app built with older plugin versions, a starting instance still notifies a first instance listening on the old `/tmp/<identifier>_si.sock` path (if it belongs to the same user), and the first instance also listens there when it can.
