---
dialog: patch
dialog-js: patch
---

Fixed `ask` and `confirm` always resolving to `false` when `okLabel` is an empty string. Empty `okLabel`/`cancelLabel` values now fall back to the default labels.
