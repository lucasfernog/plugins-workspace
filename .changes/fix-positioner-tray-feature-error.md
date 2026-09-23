---
positioner: patch
positioner-js: patch
---

Using a `Tray*` position without the `tray-icon` Cargo feature now fails with an error that names the missing feature, instead of a generic serde "invalid value" error.
