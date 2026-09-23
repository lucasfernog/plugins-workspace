---
clipboard-manager: patch
clipboard-manager-js: patch
---

If the desktop clipboard fails to initialize when the plugin is set up (for example on Linux when the display server connection is not ready yet), it is now initialized again the next time it is used, instead of failing every call for the rest of the session.
