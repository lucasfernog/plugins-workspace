---
localhost: patch
---

`Response::add_header` now compares header names case-insensitively, as HTTP does, so `add_header("content-type", ..)` in `on_request` replaces the built-in `Content-Type` header instead of sending both.
