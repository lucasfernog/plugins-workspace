---
shell: patch
shell-js: patch
---

Fixed scope argument validators and the `plugins > shell > open` regex not being fully anchored when they contain an alternation: `a|b` is now matched as `^(?:a|b)$` instead of `^a|b$`, so a value such as `a; anything` or `anything b` is no longer accepted.
