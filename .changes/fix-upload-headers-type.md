---
upload: patch
upload-js: patch
---

The `headers` argument of `upload` and `download` now accepts a plain `Record<string, string>` object in addition to a `Map<string, string>`, matching the documented examples, which previously did not type-check.
