---
notification: patch
notification-js: patch
---

On Android, other apps can no longer fake notification actions (`onAction` events) by launching the app with notification extras. The plugin now attaches a per-installation secret to the intents of its notifications and ignores intents without it. Notifications posted by a previous version of the plugin no longer emit `onAction` when tapped.
