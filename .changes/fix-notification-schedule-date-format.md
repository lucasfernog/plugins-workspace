---
notification: patch
notification-js: patch
---

Fixed `Schedule.at` notifications firing hours or days late on Android, or failing to be scheduled. The date was forwarded to the mobile plugin with nine fractional digits (and with the caller's UTC offset when set from Rust), which the Android and iOS parsers misread. It is now always sent in UTC with millisecond precision (`2026-01-01T10:00:00.123Z`).
