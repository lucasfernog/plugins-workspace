---
notification: patch
notification-js: patch
---

Fixed notifications not showing on Windows for development builds and Cargo builds outside `target\debug` / `target\release` (e.g. `--target <triple>` builds, custom profiles or a custom `CARGO_TARGET_DIR`). The app's identifier was used as the AppUserModelID even though it is only registered for installed apps.
