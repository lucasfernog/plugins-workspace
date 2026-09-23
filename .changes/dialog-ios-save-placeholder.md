---
dialog: patch
dialog-js: patch
---

On iOS, the save dialog now removes the empty placeholder file it creates in the app's Documents folder once the dialog closes, only uses the last path component of the requested file name (so it cannot point outside that folder), and no longer crashes if the picker reports no destination.
