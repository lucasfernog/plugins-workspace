---
updater: patch
updater-js: patch
---

Apply the installer arguments set with `Builder::installer_arg`/`Builder::installer_args` when the configuration has no `plugins > updater > windows` object. They were silently dropped before.
