---
store: patch
store-js: patch
---

Fixed a deadlock when a store is closed (e.g. `close()` from JavaScript) while another store is loaded, looked up with `get_store`, or saved on exit at the same time.
