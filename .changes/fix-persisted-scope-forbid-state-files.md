---
persisted-scope: patch
---

Forbid access to both state files (`.persisted-scope` and `.persisted-scope-asset`, plus their temporary files) in both the filesystem and the asset protocol scopes. Previously the asset scope state file could be written through the fs plugin when the app allowed writing to the app data directory, letting the webview grant itself asset protocol access on the next launch.
