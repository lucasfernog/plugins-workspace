---
autostart: patch
autostart-js: patch
---

`isEnabled()` on Linux now returns `false` when the user turned the autostart entry off in their desktop environment's settings (`Hidden=true` or `X-GNOME-Autostart-enabled=false`), instead of only checking that the desktop entry file exists. Calling `enable()` rewrites the entry and turns it back on.
