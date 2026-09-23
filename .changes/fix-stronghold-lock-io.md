---
stronghold: patch
stronghold-js: patch
---

Saving a stronghold no longer blocks the commands of every other stronghold while the snapshot is encrypted and written, and `Stronghold.load` now hashes the password on a blocking thread instead of an async runtime worker.
