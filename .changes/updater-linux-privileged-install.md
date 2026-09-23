---
updater: patch
updater-js: patch
---

Linux `.deb`/`.rpm` updates:

- No longer hang when the package manager writes a lot of output while installing through the zenity/kdialog password prompt.
- Stop at the first prompt the user dismisses (pkexec, zenity or kdialog) instead of asking again with the next method, and stop when the package manager itself fails under pkexec instead of retrying the install with `sudo`.
