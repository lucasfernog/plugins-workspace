---
dialog: patch
dialog-js: patch
---

**Security:** the `open` command with `directory: true` now respects the `recursive` option when it adds the picked folders to the asset protocol scope, as it already did for the fs plugin scope. Previously the asset protocol scope always allowed the folder's whole subtree. Pass `recursive: true` if you load files from subfolders of a picked folder through the asset protocol (`convertFileSrc`).
