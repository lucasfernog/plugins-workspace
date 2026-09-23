---
updater: patch
updater-js: patch
---

On macOS, extract update archives whose entries start with `./` correctly instead of nesting the app bundle in itself, and reject archive entries that would be written outside of the app bundle.
