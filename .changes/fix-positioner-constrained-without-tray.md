---
positioner: patch
positioner-js: patch
---

The `move_window_constrained` command (`moveWindowConstrained` in JS) is now registered without the `tray-icon` feature too. Without that feature it behaves like `moveWindow`, instead of failing because the command does not exist.
