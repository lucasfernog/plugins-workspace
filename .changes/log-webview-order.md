---
log: patch
log-js: patch
---

The `Webview` target now forwards log records to the webview in the order they were logged. Previously each record was emitted from its own async task, so records logged in quick succession could arrive out of order.
