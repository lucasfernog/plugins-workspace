---
store: patch
store-js: patch
---

Saving a store now writes to a temporary file and renames it over the store file, so a crash or power loss during a save no longer leaves a truncated store file behind.
