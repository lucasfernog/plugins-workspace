---
clipboard-manager: patch
clipboard-manager-js: patch
---

The desktop clipboard no longer panics when it is used after the app received `RunEvent::Exit` (for example by a command still in flight, or another plugin's exit handler): it now returns an error. A panic in the clipboard backend no longer poisons the plugin for the rest of the session either.
