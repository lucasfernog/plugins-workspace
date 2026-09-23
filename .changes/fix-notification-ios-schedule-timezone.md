---
notification: patch
notification-js: patch
---

Fixed `Schedule.at` notifications on iOS firing off by the device's UTC offset (or being rejected as scheduled in the past): the date was read in the local time zone instead of UTC.
