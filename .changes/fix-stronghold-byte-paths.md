---
stronghold: patch
stronghold-js: patch
---

Client, vault and record paths and store keys given as bytes now work in every form their types accept (for example a `Set<number>` or a `Uint16Array`, which previously failed to deserialize), and store keys can now be given as bytes instead of only as strings.
