---
store: patch
store-js: patch
---

A `Store` that was closed or replaced (with `create_new`) no longer affects the store loaded from the same path after it: `Store::close_resource` and closing its resource no longer close or unregister the new store, and its change events carry its own resource id instead of the new store's.
