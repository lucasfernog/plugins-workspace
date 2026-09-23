---
clipboard-manager: patch
clipboard-manager-js: patch
---

Android: `readText` now returns the text of any text clip (for example HTML copied from a browser) as plain text, like on desktop and iOS, instead of rejecting it, and never returns the literal string `"null"`.
