---
opener: patch
opener-js: patch
---

Fixed the `revealItemInDir` fallback on Linux and BSD for systems without `org.freedesktop.FileManager1`: the `org.freedesktop.portal.OpenURI` `OpenDirectory` call used swapped service and interface names and the wrong signature, so it always failed. If the portal is unavailable too, the parent directory is now opened with the default file manager, and the original error is reported when every method fails. The command also no longer blocks an async runtime thread.
