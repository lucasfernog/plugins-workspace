---
single-instance: patch
---

On Windows, arguments that contain `|` (for example `--filter=a|b` or deep-link URLs) are no longer split into several arguments when they are forwarded to the first instance. The second instance now sends a NUL separated payload and only falls back to the previous `|` separated format when the first instance runs an older plugin version, so different app versions running side by side still work together.
