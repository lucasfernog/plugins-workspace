---
shell: patch
shell-js: patch
---

Fixed a child spawned from JavaScript becoming impossible to `kill()` or `write()` to, and no longer being killed on exit, when it reused the pid of an earlier child whose `close` event was delivered late. Children whose exit status could not be read are also no longer kept in the plugin state forever.
