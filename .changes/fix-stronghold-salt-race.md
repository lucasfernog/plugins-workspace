---
stronghold: patch
stronghold-js: patch
---

Fixed a race in `Builder::with_argon2` where concurrent `Stronghold.load` calls on a fresh install could each write a different salt, making snapshots saved with the first salt impossible to open. The salt file is now also written atomically, so a crash cannot leave a truncated salt behind.
