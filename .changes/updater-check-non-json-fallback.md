---
updater: patch
updater-js: patch
---

`check()` now moves on to the next endpoint when an endpoint answers with a successful status but a body that is not JSON (for example a captive portal or CDN error page), instead of failing right away. The status code of endpoints that do not answer with a successful status is now logged.
