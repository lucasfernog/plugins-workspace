---
shell: minor
shell-js: minor
---

Added `ExitStatus::signal()`, which returns the signal that terminated a process run with the Rust `Command::status()` or `Command::output()`, so a signal-killed process can be told apart from one without an exit code. The JavaScript API already reported it.
