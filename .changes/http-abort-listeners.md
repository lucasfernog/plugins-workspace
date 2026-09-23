---
http: patch
http-js: patch
---

`fetch` now removes the `abort` listeners it adds to the request's `signal` once the response is received and once the body is read, cancelled or dropped, so a long-lived `AbortSignal` shared by many requests no longer accumulates listeners that all send IPC calls when it is finally aborted.
