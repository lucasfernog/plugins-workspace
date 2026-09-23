---
persisted-scope: patch
---

Persist the scope when a path is forbidden at runtime too. Previously only allowing a path saved the state, so a runtime `forbid_*` call was lost on restart unless a later allow happened.
