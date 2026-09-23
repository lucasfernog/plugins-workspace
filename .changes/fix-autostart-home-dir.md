---
autostart: patch
autostart-js: patch
---

Return an error instead of panicking when the home directory can't be resolved on macOS and Linux, and create `~/.config/autostart` along with any missing parent directory on Linux, so enabling autostart no longer fails when `~/.config` doesn't exist.
