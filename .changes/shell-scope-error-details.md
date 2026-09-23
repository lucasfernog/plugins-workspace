---
shell: patch
shell-js: patch
---

When `execute` or `spawn` is called for a program that is in the shell scope but with arguments (or a sidecar flag) that do not match it, the error now says which argument failed and why, instead of the generic `program not allowed on the configured shell scope` message, which is still returned for programs that are not in the scope or are denied.
