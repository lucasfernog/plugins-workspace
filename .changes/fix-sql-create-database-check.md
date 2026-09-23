---
sql: patch
sql-js: patch
---

`load` no longer tries to create the database when checking whether it exists fails (for example a MySQL or PostgreSQL server rejecting the check). It connects to the database directly instead, so a failed check neither creates an unwanted database nor hides the real error behind a failed creation.
