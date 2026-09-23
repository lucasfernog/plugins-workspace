---
nfc: patch
nfc-js: patch
---

Fixed the `mime_type`, `tech_list` and `uri.path_prefix` filters of the Rust `ScanKind` being silently ignored on Android. They were serialized in snake case (`mime_type`, `tech_list`, `path_prefix`) instead of the `mimeType`, `techLists` and `pathPrefix` names the native plugin reads.
