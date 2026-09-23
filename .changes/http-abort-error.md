---
http: patch
http-js: patch
---

An aborted `fetch` now always rejects with a `DOMException` named `AbortError` (still with the `Request cancelled` message), like the standard `fetch`, so `error.name === 'AbortError'` checks work. It used to reject with a plain `Error`, or with the `Request canceled` string when the abort happened while waiting for the response, and the body stream was errored with a string.
