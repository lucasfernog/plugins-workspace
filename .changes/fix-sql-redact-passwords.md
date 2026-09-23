---
sql: patch
sql-js: patch
---

The `invalid connection url` and `database … not loaded` error messages now mask the password of MySQL and PostgreSQL connection strings, so credentials are no longer sent back to the frontend or written to logs.
