---
global-shortcut: patch
global-shortcut-js: patch
---

Shortcuts registered from JavaScript are now unregistered when the webview that registered them reloads, navigates to another page or has its window destroyed. Their handler is an IPC channel into that page and could never run again, and registering the same shortcut from the new page failed with "already registered".
