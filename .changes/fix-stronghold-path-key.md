---
stronghold: patch
stronghold-js: patch
---

Different spellings of the same snapshot path (for example `dir/./vault.hold`, `dir/sub/../vault.hold` or a path through a symlink) now refer to the same stronghold instance instead of separate instances that overwrite each other's changes when saved.
