---
shell: minor
shell-js: minor
---

Added the optional `env` and `cwd` fields to shell scope entries, to restrict what the webview can pass as the `env` and `cwd` options of `Command.create()` / `Command.sidecar()`. `env` is `true` (any variable, the default), `false` (none) or a list of the variable names that may be set; `cwd` is `true` (the default) or `false`. Without them the webview can, for example, set `PATH` or `LD_PRELOAD` for an otherwise tightly scoped command and make it run other code, so restrict them for commands that untrusted content can call.
