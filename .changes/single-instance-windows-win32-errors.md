---
single-instance: patch
---

On Windows, failures to create the named mutex or the hidden message window are now logged and the app launches normally, instead of storing and later releasing a null handle or silently running without a message window.
