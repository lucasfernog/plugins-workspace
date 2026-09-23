---
upload: patch
upload-js: patch
---

`download` now writes to a temporary file next to the destination and moves it into place only once the whole response has been received. A failed download no longer leaves a partial file behind or destroys an existing file at the destination path.
