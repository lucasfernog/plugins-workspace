---
opener: patch
opener-js: patch
---

Fixed the `openPath` scope check allowing a path to be opened with a program that is only allowed by a *different* scope entry. The path and the requested `with` program must now be allowed by the same `path` entry, e.g. with `[{ "path": "$DOWNLOAD/**" }, { "path": "$APPCONFIG/app.toml", "app": "notepad" }]`, `openPath('$DOWNLOAD/file.txt', 'notepad')` is now rejected.
