---
cli: patch
cli-js: patch
---

Fixed a panic when reading the matches of a flag (an argument without `takesValue`) configured with `multiple: true`. Such a flag can now be repeated, for example `-vvv`, and `occurrences` reports how many times it was passed.
