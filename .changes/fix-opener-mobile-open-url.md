---
opener: patch
opener-js: patch
---

Fixed `openUrl` on iOS resolving successfully for an invalid URL or a URL no app can open, and crashing the app when `inAppBrowser` was used with a non-`http(s)` URL. On Android and iOS, `inAppBrowser` now opens non-`http(s)` URLs (e.g. `mailto:`) with their default app, since Custom Tabs and `SFSafariViewController` only support web URLs.
