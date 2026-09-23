---
sql: patch
sql-js: patch
---

Fixed MySQL `select` failing with `unsupported datatype` for `TINYBLOB`, `BINARY`, `VARBINARY`, `DECIMAL`, `BIT` and `SET` columns. Binary columns are returned as byte arrays like `BLOB`, `DECIMAL` like PostgreSQL `NUMERIC` (a number, or a string when it cannot be represented as one), `BIT` as a number and `SET` as its comma-separated string.
