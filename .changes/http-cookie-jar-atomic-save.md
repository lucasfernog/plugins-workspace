---
http: patch
http-js: patch
---

The cookie jar file is now written atomically (to a temporary file that is then renamed) and its writes are serialized, so overlapping saves or a crash during a save can no longer leave a truncated `.cookies` file, which dropped every cookie on the next start.
