---
updater: patch
updater-js: patch
---

On Linux, only set `SSL_CERT_FILE`/`SSL_CERT_DIR` to the Debian certificate locations when they exist, so that checking for updates no longer points TLS at a missing file on other distributions.
