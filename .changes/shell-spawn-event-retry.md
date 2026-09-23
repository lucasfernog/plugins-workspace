---
shell: patch
shell-js: patch
---

A child spawned from JavaScript no longer hangs, and its event forwarding task no longer grows its memory forever, once its events cannot be delivered to the webview (for example after the window was closed). Delivery is retried for about 5 seconds, then the remaining events are dropped while the child's output keeps being drained.
