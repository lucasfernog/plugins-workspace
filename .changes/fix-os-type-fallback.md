---
os: patch
os-js: patch
---

`type_()` now compiles on targets other than Linux, the BSDs, Windows, macOS, iOS and Android (e.g. illumos or Solaris) and reports `OsType::Linux` there, as it already did for the BSDs.
