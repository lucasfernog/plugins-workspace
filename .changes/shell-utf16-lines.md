---
shell: patch
shell-js: patch
---

Fixed `Command.spawn()` output with `encoding: 'utf-16le'` or `'utf-16be'` turning into garbage after the first line: the output was split into lines on the raw `\n` byte, which is only half of a UTF-16 newline. It is now decoded first and then split into lines.
