---
store: patch
store-js: patch
---

A store whose resource was removed from the resources table without being closed can be loaded again instead of failing with a bad resource id error forever, and `getStore` returns `null` for it instead of a dead handle.
