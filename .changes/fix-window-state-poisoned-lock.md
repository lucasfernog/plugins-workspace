---
window-state: patch
window-state-js: patch
---

A panic while the window state cache was locked no longer makes every later window move, resize or close event panic on the main thread: the plugin now keeps using the cache after a poisoned lock.
