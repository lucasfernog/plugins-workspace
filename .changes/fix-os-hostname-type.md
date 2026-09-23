---
os: patch
os-js: patch
---

Fixed the return type of `hostname()`: it resolves to `string`, never `null`, matching the Rust command.
