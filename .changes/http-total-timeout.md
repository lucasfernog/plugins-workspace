---
http: minor
http-js: minor
---

Added the `timeout` client option to `fetch`: a total timeout in milliseconds, from sending the request until its body is fully read. Previously only `connectTimeout` existed, so a stalled server made a request hang forever unless it was aborted.
