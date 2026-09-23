---
localhost: patch
---

The localhost server now answers `404` to request paths that contain a `..` segment. In `tauri dev` (with `devUrl` and a `frontendDist` directory) the asset resolver reads files from disk, and such paths could read files outside `frontendDist`.
