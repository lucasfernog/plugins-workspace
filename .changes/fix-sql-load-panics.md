---
sql: patch
sql-js: patch
---

Loading a SQLite database no longer panics when the app config directory cannot be resolved or created, or when the database path is not valid UTF-8. `load` now rejects with an error instead.
