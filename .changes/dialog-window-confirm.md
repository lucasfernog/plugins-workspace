---
dialog: patch
dialog-js: patch
---

Fixed the injected `window.confirm` override always rejecting: it invoked a `confirm` command that does not exist. It now shows an `Ok`/`Cancel` message dialog through the `message` command (`dialog:allow-message`) and resolves to `true` when `Ok` is pressed. Tauri has no synchronous IPC, so the override still returns a `Promise<boolean>` that must be awaited.
