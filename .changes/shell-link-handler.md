---
shell: patch
shell-js: patch
---

The script the plugin injects to open `<a target="_blank">` links with the system browser now leaves alone clicks another handler already cancelled (`preventDefault()`), and logs a failed open (for example when `shell:allow-open` is not granted) instead of leaving an unhandled promise rejection.
