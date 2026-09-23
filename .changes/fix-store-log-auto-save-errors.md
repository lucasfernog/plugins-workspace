---
store: patch
store-js: patch
---

Log an error when an automatic save of a store fails (debounced auto save, `autoSave: 0`, or the pending save applied when a store is dropped), instead of silently discarding it.
