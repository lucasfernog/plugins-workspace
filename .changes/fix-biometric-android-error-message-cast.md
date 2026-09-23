---
biometric: patch
biometric-js: patch
---

Fixed a possible `ClassCastException` on Android when the system reports an authentication error message that is not a plain `String`, which ended the prompt as `systemCancel` instead of the actual error.
