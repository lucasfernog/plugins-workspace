---
stronghold: patch
stronghold-js: patch
---

Calling `Stronghold.load` again for a snapshot that is already loaded with the same password (for example after a webview reload or from a second window) now reuses the loaded instance instead of replacing it with the state on disk, which discarded unsaved changes and broke the clients loaded through other handles.
