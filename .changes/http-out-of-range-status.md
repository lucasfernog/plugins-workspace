---
http: patch
http-js: patch
---

`fetch` no longer rejects with a `RangeError` (leaking the response body) when the server answers with a status the `Response` constructor refuses, outside of 200-599, such as the non-standard `999`: the response reports the actual `status`, and `ok` is `false`.
