---
websocket: minor
websocket-js: minor
---

Added an opt-in URL scope for `connect`: `allow` / `deny` URL patterns on the `websocket:default` or `websocket:allow-connect` permission. Without any entry every URL is still allowed. Once an `allow` entry is configured only matching URLs can be opened, and `deny` entries always take precedence.
