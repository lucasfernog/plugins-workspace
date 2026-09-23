---
deep-link: patch
deep-link-js: patch
---

On Windows and Linux, deep links for a scheme configured with uppercase letters (for example `MyApp`) are now recognized: URL schemes are case-insensitive and were compared against the lowercased scheme of the parsed URL, so `get_current` stayed empty and no event was emitted.
