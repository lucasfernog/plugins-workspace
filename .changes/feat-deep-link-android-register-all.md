---
deep-link: minor
deep-link-js: minor
---

Added `DeepLink::register_all` on Android, which only existed on the other platforms, so cross-platform code calling it compiles for Android too. Like on macOS and iOS it returns `Error::UnsupportedPlatform` when desktop schemes are configured.
