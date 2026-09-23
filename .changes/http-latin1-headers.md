---
http: patch
http-js: patch
---

A response header value that is not valid UTF-8 (for instance a Latin-1 `Content-Disposition` filename), or that contains characters the JavaScript `Headers` class rejects, no longer fails the whole request: it is decoded byte by byte, as browsers do.
