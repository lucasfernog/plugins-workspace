---
deep-link: patch
deep-link-js: patch
---

On Linux, `register` and `unregister` now log a warning when `xdg-mime` or `update-desktop-database` runs but exits with a failure status, which was silently ignored.
