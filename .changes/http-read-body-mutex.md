---
http: patch
http-js: patch
---

Fixed undefined behavior when `fetch_read_body` is called concurrently for the same response: the response body is now locked while a chunk is read instead of being mutably aliased through an `unsafe` pointer cast.
