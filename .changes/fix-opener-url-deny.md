---
opener: patch
opener-js: patch
---

Fixed `openUrl` deny scope entries being bypassable: a deny entry without an `app` now forbids the URL for every `with` program (not only the default application, matching how path deny entries work), and deny entries are also matched against the normalized URL, so case, default port and trailing dot variants of a denied URL are rejected too.
