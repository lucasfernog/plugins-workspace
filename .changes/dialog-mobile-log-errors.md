---
dialog: patch
dialog-js: patch
---

On Android and iOS, a native dialog failure (for example a rejected picker or an unreadable result) is still reported as a cancelled dialog, but it is now logged with `log::error!` instead of being silently discarded. A cancelled iOS picker is no longer treated as a deserialization failure internally.
