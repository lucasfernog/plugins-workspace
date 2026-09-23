---
stronghold: patch
stronghold-js: patch
---

`Vault.remove` now rejects a location created with `Location.counter` with a clear error message instead of an opaque deserialization error, and rejects a location that points to a different vault instead of removing the record with the same path from the vault it was called on.
