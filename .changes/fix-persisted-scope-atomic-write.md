---
persisted-scope: patch
---

Write the persisted scope files atomically (temporary file and rename) and serialize concurrent writes. A state file that can't be read is now logged and left untouched instead of being replaced with an empty scope.
