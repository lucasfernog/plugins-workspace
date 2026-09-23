---
positioner: patch
positioner-js: patch
---

With the `tray-icon` feature enabled, `WindowExt::move_window` and `WindowExt::move_window_constrained` no longer panic when the plugin is not registered. Screen positions now work without the plugin, tray positions return an error, and `on_tray_event` logs a warning instead of panicking.
