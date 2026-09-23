---
global-shortcut: patch
global-shortcut-js: patch
---

`unregister_all` (and the JavaScript `unregisterAll`) no longer holds the plugin's internal lock while waiting for the main thread, which could deadlock when a hotkey event was being dispatched at the same time. When unregistering some of the shortcuts fails, `unregister`, `unregister_multiple` and `unregister_all` now still unregister the others and only forget the ones that were actually unregistered, so `is_registered` keeps matching what is registered with the OS.
