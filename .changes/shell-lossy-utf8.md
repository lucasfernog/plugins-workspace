---
shell: patch
shell-js: patch
---

Output that is not valid UTF-8 (with no `encoding` set) is now decoded with `U+FFFD` replacement characters instead of being lost: `Command.execute()` used to reject and return neither the output nor the exit code, and `spawn()` replaced each such line with an `error` event.
