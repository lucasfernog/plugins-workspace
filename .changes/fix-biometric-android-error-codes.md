---
biometric: patch
biometric-js: patch
---

On Android, `authenticate` now rejects with the `biometryNotAvailable` code instead of no code when the system requires a security update before biometrics can be used.
