---
deep-link: patch
deep-link-js: patch
---

On Linux, `register` and `unregister` no longer strip the quotes around the executable path in the `Exec` line of the handler's `.desktop` file when they update it, which broke the remaining schemes when the path contains spaces. `register` also no longer rewrites the file when nothing changed.
