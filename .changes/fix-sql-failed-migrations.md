---
sql: patch
sql-js: patch
---

Fixed migrations being skipped after they failed to apply: they were forgotten before running, so the next `load` of the same database succeeded without migrating it. Failed migrations are now tried again on the next `load`, and a concurrent `load` of the same database waits for its migrations to finish.
