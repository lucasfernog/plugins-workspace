---
opener: patch
opener-js: patch
---

Fixed `revealItemInDir` / `reveal_item_in_dir` / `reveal_items_in_dir` on Android and iOS returning an I/O error for a missing path instead of the documented `UnsupportedPlatform` error.
