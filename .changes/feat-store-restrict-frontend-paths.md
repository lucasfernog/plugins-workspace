---
store: minor
store-js: minor
---

Add `Builder::restrict_frontend_paths`, an opt-in that makes the `load` and `get_store` commands reject store paths that are absolute, contain `..` or have no file name, so frontend code cannot read or overwrite files outside the app data directory. It is disabled by default for backwards compatibility.
