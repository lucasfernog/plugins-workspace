---
stronghold: patch
stronghold-js: patch
---

`Stronghold.loadClient` now resolves when the client is already loaded (for example by another window, or created in this session) instead of rejecting. Previously the documented `loadClient` → `createClient` fallback then replaced the loaded client with an empty one, and the next save wiped the client's data.
