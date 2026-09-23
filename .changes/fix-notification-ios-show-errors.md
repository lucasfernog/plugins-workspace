---
notification: patch
notification-js: patch
---

On iOS, showing or scheduling a notification now waits for the system to accept it and reports its errors (e.g. when the app is not allowed to post notifications) instead of resolving right away and dropping them. Error messages also include the underlying reason (e.g. `Scheduled time must be *after* current time`) instead of an internal case name.
