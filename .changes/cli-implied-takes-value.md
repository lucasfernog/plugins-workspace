---
cli: patch
cli-js: patch
---

Arguments that set `index`, `possibleValues`, `numberOfValues`, `minValues` or `maxValues` now take a value even if `takesValue` is not set. Previously, such a configuration made argument parsing panic in debug builds.
