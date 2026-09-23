---
http: patch
http-js: patch
---

With the `unsafe-headers` feature, a `Content-Length` or `Accept-Encoding` header set by the frontend is no longer sent twice alongside the default `Content-Length: 0` (body-less `POST`/`PUT`) or `Accept-Encoding: identity` (`Range` requests).
