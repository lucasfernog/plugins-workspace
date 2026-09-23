---
shell: patch
shell-js: patch
---

`deny` entries of the `shell:allow-execute` and `shell:allow-spawn` scopes (and the global shell scope) are now enforced: a command whose `name` matches a denied entry is rejected even when an `allow` entry also matches it. Previously deny entries were ignored.
