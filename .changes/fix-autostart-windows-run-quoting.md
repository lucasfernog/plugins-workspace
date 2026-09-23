---
autostart: patch
autostart-js: patch
---

Quote the executable path, and any startup argument containing spaces or quotes, in the Windows `Run` registry value. Previously an unquoted path with spaces (e.g. a per-user install under `%LOCALAPPDATA%\My App`) could be resolved to the wrong program, and arguments with spaces were split.
