---
notification: patch
notification-js: patch
---

Fixed Android notification channels:

- The `lightColor` of a channel is now applied; it was read from a misnamed field.
- `channels()` now reports the light color, reports a missing sound as `null` instead of the string `"null"`, and always reports an importance, which previously broke the Rust `Notification::list_channels` for channels with the maximum or an unspecified importance.
