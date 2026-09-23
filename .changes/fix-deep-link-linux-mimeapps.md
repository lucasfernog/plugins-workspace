---
deep-link: patch
deep-link-js: patch
---

On Linux, `unregister` only rewrites the user's `~/.config/mimeapps.list` when the app actually was the default handler of the scheme, keeping the rest of the file (comments included) untouched, and also removes the app when the handler is written as a `;`-separated list. It no longer fails when that file contains lines it could not parse.
