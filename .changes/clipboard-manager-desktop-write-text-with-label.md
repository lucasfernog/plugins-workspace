---
clipboard-manager: minor
clipboard-manager-js: minor
---

Added `Clipboard::write_text_with_label` on desktop, so cross-platform Rust code no longer needs to `cfg`-gate it. The label is only used on Android; on desktop it behaves like `write_text`.
