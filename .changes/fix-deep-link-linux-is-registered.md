---
deep-link: patch
deep-link-js: patch
---

On Linux, `is_registered` no longer reports `true` when another app whose handler file name merely ends with this app's one (for example `myapp-handler.desktop` for an app named `app`) is the scheme's handler.
