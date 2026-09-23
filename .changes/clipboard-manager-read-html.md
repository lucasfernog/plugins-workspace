---
clipboard-manager: minor
clipboard-manager-js: minor
---

Added `readHtml` (`Clipboard::read_html` in Rust) to read HTML from the clipboard on desktop, guarded by the new `clipboard-manager:allow-read-html` permission. It returns an error on Android and iOS.
