---
shell: patch
shell-js: patch
---

`Child.write()` no longer blocks the main thread, nor every other shell command, while the child process does not read its stdin. Previously a write that filled the pipe froze the app, and `kill()` and the exit cleanup waited on it forever. Writes made through the same `Child` still reach the process in order, even when they are not awaited.
