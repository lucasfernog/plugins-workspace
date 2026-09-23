---
fs: patch
fs-js: patch
---

Fixed a leak of iOS security-scoped resource access: every fs command that received a `file://` URL started accessing the resource and never stopped it. The access is now stopped when the command finishes (or when the returned file handle is closed). Resources started with `startAccessingSecurityScopedResource` are still only stopped by `stopAccessingSecurityScopedResource`.
