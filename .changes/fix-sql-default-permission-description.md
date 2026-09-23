---
sql: patch
sql-js: patch
---

Corrected the description of the `sql:default` permission set: it does not enable `execute`, and it is not a read-only mode, because `select` runs any SQL statement it is given. The permissions it grants are unchanged.
