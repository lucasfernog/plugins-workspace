---
clipboard-manager: patch
clipboard-manager-js: patch
---

`Clipboard::read_image` now returns an owned `Image<'static>` (it no longer borrows the clipboard) and no longer copies the image data twice.
