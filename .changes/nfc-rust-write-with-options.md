---
nfc: minor
nfc-js: minor
---

Added `Nfc::write_with_options` and `WriteOptions` to the Rust API. They set the scan `kind` used to find the tag to write to, so a write works on Android without a kept-alive `scan` session, and the messages displayed in the iOS UI, like the JavaScript `write` options.
