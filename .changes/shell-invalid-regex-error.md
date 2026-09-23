---
shell: patch
shell-js: patch
---

An invalid `validator` regex in a shell scope no longer panics while handling an `execute` or `spawn` call; the call is rejected with an error instead. An invalid `plugins > shell > open` regex now makes the plugin setup return an error instead of panicking.
