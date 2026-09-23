---
log: patch
log-js: patch
---

Fixed the caller location (the `webview::<location>` log target) on WebKit (macOS, iOS and Linux) when logging from an anonymous function such as an arrow function callback. Anonymous frames were skipped, so the location of the next named function up the stack was reported. They are now reported as `<anonymous>@<file>:<line>:<column>`, as on Chromium.
