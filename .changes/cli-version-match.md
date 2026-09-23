---
cli: patch
cli-js: patch
---

The `version` match returned for `--version` now has the version text (for example `app 1.0.0\n`) as its `value` instead of `null`, like the `help` match does with the help text. Both matches now report `occurrences: 1` instead of `0`.
