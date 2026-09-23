---
notification: patch
notification-js: patch
---

`pending()` now includes the `schedule` of each notification on iOS, which also fixes the Rust `Notification::pending` method always failing there. Interval schedules are reported as `interval`; `at` and `every` schedules are reported as `at` with the date of the next delivery.
