---
store: patch
store-js: patch
---

Fixed a deadlock when a Rust `store://change` listener reads or writes a store, or loads another one: change events are now emitted after the store's locks are released.
