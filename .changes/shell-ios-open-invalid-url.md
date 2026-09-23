---
shell: patch
shell-js: patch
---

On iOS, `open` now rejects a URL that cannot be parsed instead of silently succeeding, and opens URLs from the main thread as UIKit requires.
