---
shell: patch
shell-js: patch
---

With raw output (`encoding: 'raw'` in JavaScript, `set_raw_out(true)` in Rust), a failing read of the child's stdout or stderr no longer loops forever sending `Error` events; the error is reported once and reading that stream stops. Interrupted reads are retried silently.
