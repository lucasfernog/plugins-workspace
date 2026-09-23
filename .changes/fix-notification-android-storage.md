---
notification: patch
notification-js: patch
---

Fixed scheduled notifications on Android, which were never saved by the plugin:

- `pending()` now lists them and `cancelAll()` cancels their alarms.
- They are restored after a reboot, and `onNotificationReceived` fires when they are delivered.
- `onAction` now receives the notification in its `notification` field instead of `null`.
- Repeating (`every` and repeating `at`) notifications stay pending after they fire.
