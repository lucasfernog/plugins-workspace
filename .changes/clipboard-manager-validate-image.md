---
clipboard-manager: patch
clipboard-manager-js: patch
---

`write_image` (`writeImage` in JS) now rejects an image whose RGBA buffer is not exactly `width * height * 4` bytes long instead of panicking inside the clipboard backend (or reading out of bounds on macOS).
