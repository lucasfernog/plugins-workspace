---
biometric: patch
biometric-js: patch
---

Fixed the iOS biometry status being computed once when the plugin loads: `checkStatus` and `authenticate` now evaluate it on every call, so enrolling Face ID/Touch ID, removing the enrolment or a biometry lockout while the app runs is picked up without restarting the app.
