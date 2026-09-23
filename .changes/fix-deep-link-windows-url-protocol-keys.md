---
deep-link: patch
deep-link-js: patch
---

On Windows, `register` no longer takes over a class that is not a URL protocol (for example a ProgID such as `exefile`), and `unregister` only deletes `Software\Classes` keys that are URL protocols (they have a `URL Protocol` value), so it can no longer wipe file associations.
