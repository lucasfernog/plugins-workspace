---
clipboard-manager: patch
clipboard-manager-js: patch
---

iOS: `readText` now rejects with "Clipboard content is not text" instead of "Clipboard is empty" when the clipboard holds non-text content such as an image.
