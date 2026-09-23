---
http: patch
http-js: patch
---

Complete the list of forbidden request headers dropped when the `unsafe-headers` feature is disabled, per the Fetch spec: `Keep-Alive`, `Cookie2`, `Access-Control-Request-Private-Network`, and `X-HTTP-Method`, `X-HTTP-Method-Override` and `X-Method-Override` when they name the `CONNECT`, `TRACE` or `TRACK` method.
