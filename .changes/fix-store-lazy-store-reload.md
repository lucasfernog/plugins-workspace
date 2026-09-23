---
store: patch
store-js: patch
---

`LazyStore` no longer caches a failed load, so the next call tries to load the store again, and it loads the store again when used after `close()` instead of failing with an invalid resource id.
