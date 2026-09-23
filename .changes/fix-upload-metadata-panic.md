---
upload: patch
upload-js: patch
---

`upload` no longer panics when the file's metadata cannot be read; it now rejects with the underlying I/O error instead of a generic "task panicked" message.
