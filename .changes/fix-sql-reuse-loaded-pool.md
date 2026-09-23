---
sql: patch
sql-js: patch
---

Calling `load` for a database that is already loaded now reuses its connection pool instead of opening a new one and discarding the pool other callers were using. A database whose pool was closed is still reconnected by `load`. `DbPool` now implements `Clone`.
