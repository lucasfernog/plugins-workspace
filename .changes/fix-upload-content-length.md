---
upload: patch
upload-js: patch
---

`upload` now ignores a user-supplied `Content-Length` header. The plugin always sets it from the file size, and the user value was appended as a second, conflicting header that could make the request fail.
