---
opener: patch
opener-js: patch
---

Fixed `revealItemInDir` on Windows silently succeeding when `SHOpenFolderAndSelectItems` fails, and never using its `ShellExecuteExW` fallback for the "file not found" failure. It also no longer leaves COM initialized as a single-threaded apartment on the async runtime thread that ran it.
