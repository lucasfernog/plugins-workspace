---
deep-link: patch
deep-link-js: patch
---

`register` and `unregister` now reject protocols that are not valid URI schemes (an ASCII letter followed by ASCII letters, digits, `+`, `-` or `.`), and `is_registered` returns `false` for them. Previously the raw string was used to build a registry key path on Windows (so `unregister` could delete arbitrary `Software\Classes` keys such as `.txt`) and was written unescaped into the `.desktop` file on Linux (so a new line could inject extra keys).
