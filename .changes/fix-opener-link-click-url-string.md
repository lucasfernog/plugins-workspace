---
opener: patch
opener-js: patch
---

Fixed links opened by the injected click handler (`target="_blank"`, Ctrl/Shift-click) failing with a `DataCloneError`: the handler passed a `URL` object to `open_url`, which the IPC could not clone. It now passes the URL string.
