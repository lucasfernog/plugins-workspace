---
opener: patch
opener-js: patch
---

Fixed `openPath` / `Opener::open_path` always failing on Android and iOS. On Android the file is now shared through a `FileProvider` (declared by the plugin as `app.tauri.opener.OpenerFileProvider` with the `${applicationId}.opener.fileprovider` authority) and opened with the default app for its type. On iOS it is previewed with Quick Look, from which it can be shared or opened in another app. The `with` argument is ignored on mobile.
