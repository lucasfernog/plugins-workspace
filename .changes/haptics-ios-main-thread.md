---
"haptics": patch
"haptics-js": patch
---

On iOS, `impactFeedback`, `notificationFeedback` and `selectionFeedback` now use the UIKit feedback generators on the main thread. They were called from Tauri's IPC queue, which UIKit does not support and which could produce no feedback.
