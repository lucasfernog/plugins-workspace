---
notification: patch
notification-js: patch
---

On iOS, notification attachments are now copied before being handed to the system, which moves attachment files into its own store. Previously any file the app could access (e.g. a database) could be moved out of place, effectively deleted, by passing it as an attachment.
