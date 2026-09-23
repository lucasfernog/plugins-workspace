---
sql: patch
sql-js: patch
---

`close` no longer blocks `load` calls while it waits for in-flight queries of the closed pools to finish.
