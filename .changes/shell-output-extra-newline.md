---
shell: patch
shell-js: patch
---

`Command::output()` in Rust no longer adds an extra `\n` after every line (or, with `set_raw_out(true)`, after every chunk it read). `stdout` and `stderr` now hold exactly what the process wrote, as the JavaScript `execute()` already did: a process printing `a\n` used to give `a\n\n`, and raw binary output was corrupted.
