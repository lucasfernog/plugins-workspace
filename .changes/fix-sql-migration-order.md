---
sql: patch
sql-js: patch
---

Migrations now run in ascending `version` order, as documented, instead of the order they were passed to `Builder::add_migrations`.
