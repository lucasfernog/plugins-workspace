---
store: patch
store-js: patch
---

The `autoSave` load option now accepts fractional numbers of milliseconds, and an invalid value (a negative or too large number) is rejected with a clear error instead of "data did not match any variant of untagged enum AutoSave".
