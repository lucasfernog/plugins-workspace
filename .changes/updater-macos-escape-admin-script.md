---
updater: patch
updater-js: patch
---

On macOS, escape the app bundle path and the extracted update path in the shell command run with administrator privileges when the app directory is not writable. Paths containing `'`, `"` or `\` could break out of the quoting and run other commands as root.
