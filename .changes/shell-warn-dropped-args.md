---
shell: patch
shell-js: patch
---

`execute` and `spawn` now log a warning when a scoped command is called with more arguments than its scope allows (including any argument for `args: false`), or with a value at a fixed argument's position that differs from it. Those arguments are still ignored, as before; a future major version will reject such calls.
