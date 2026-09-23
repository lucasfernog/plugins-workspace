---
cli: patch
cli-js: patch
---

Fixed `occurrences` of a `multiple` argument counting its values instead of the number of times it was passed. For example, `--arg 1 --arg 2 3` now reports 2 occurrences, as documented. The count also stops at 255 instead of wrapping around.
