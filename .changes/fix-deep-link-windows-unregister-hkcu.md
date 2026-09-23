---
deep-link: patch
deep-link-js: patch
---

On Windows, `unregister` now removes the per-user registration (the one `register` writes) before the per-machine one, so it is cleaned up even when removing the per-machine key fails for lack of admin rights.
