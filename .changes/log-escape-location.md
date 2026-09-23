---
log: patch
log-js: patch
---

Control characters such as line breaks in the `location` and `file` sent by the webview's `log` command are now escaped (e.g. `\n`) before they are written into the log record's target and file, so a webview cannot forge log lines through them.
