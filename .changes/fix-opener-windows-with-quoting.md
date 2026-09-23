---
opener: patch
opener-js: patch
---

Fixed `openUrl` / `openPath` with a `with` program on Windows passing the URL or path unquoted on the program's command line: paths with spaces were split into several arguments, and a URL with spaces could inject extra arguments into the program. The URL or path is now quoted as a single argument.
