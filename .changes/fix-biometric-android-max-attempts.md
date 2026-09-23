---
biometric: minor
biometric-js: minor
---

Implemented the Android `maxAttemps` authentication option, which was parsed but never applied: once the given number of biometric attempts fails, the prompt is dismissed and `authenticate` rejects with the `authenticationFailed` code. When the option is not set the prompt keeps its previous behavior (no limit besides the system lockout). Added the correctly spelled `maxAttempts` option and deprecated `maxAttemps`.
