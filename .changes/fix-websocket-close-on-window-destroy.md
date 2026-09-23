---
websocket: patch
websocket-js: patch
---

Connections are now closed (with a `1001 Going Away` close frame) when the window that opened them is destroyed, instead of staying open until the server hangs up.
