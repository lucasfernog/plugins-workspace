---
window-state: patch
window-state-js: patch
---

Errors that were silently ignored are now logged with the `log` crate: a state file that can't be read or parsed (a missing file is still silent), and failures restoring a window when it is created, reading its state when it is closed, or saving the state on exit.
