---
shell: patch
shell-js: patch
---

The `env` spawn option now accepts `null` in its TypeScript type, which clears the environment the process inherits, as documented and already supported at runtime. Its documentation also says that the variables are added to the inherited environment, and how a scope entry's `env` and `cwd` fields restrict these options.
