---
http: patch
http-js: patch
---

`fetch` now honors the `signal` of a `Request` passed as its input, like the standard `fetch`: `fetch(new Request(url, { signal }))` could not be aborted.
