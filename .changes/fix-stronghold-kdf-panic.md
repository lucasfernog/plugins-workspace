---
stronghold: patch
stronghold-js: patch
---

`Stronghold.load` now rejects instead of panicking when the salt file of `Builder::with_argon2` cannot be read or written, or is not 32 bytes long. The salt file's parent directory is now created when it does not exist yet.
