---
log: patch
log-js: patch
---

Fixed a memory leak on iOS where every record written to the `Stdout`/`Stderr` target from a Rust thread without an autorelease pool (e.g. async runtime workers) kept its message string alive until the thread exited.
