---
global-shortcut: patch
global-shortcut-js: patch
---

Registering a list of shortcuts (`register_multiple`, `on_shortcuts` and the JavaScript `register([...])`) is now all-or-nothing: if one shortcut fails to register, the ones registered by that call are unregistered again, instead of staying registered while the call reports an error. The plugin also no longer holds its internal lock while waiting for the main thread to register the shortcuts, which could deadlock when a hotkey event was being dispatched at the same time.
