---
nfc: patch
nfc-js: patch
---

An unknown scan `kind` on Android now throws an `IllegalArgumentException` instead of a `java.lang.Error`, so it is handled like any other invalid argument.
