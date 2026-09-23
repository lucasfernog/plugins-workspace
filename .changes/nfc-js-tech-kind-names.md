---
nfc: patch
nfc-js: patch
---

`scan` and `write` now send the `techLists` filter of an `ndef` scan kind as technology names, the format the Android plugin expects, instead of the numeric values of the `TechKind` enum.
