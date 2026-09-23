---
store: patch
store-js: patch
---

`Store::save` no longer panics (poisoning the store's lock) when the store path has no file name, such as `/` or `C:\`; it returns an error instead.
