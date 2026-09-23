---
single-instance: patch
---

On macOS, a process that connects to the first instance's socket and never closes the connection can no longer block the notifications of later instances: each connection is now read separately, with a 5 second timeout and a 4 MiB size limit. Messages that are not valid UTF-8 are now converted lossily instead of being dropped.
