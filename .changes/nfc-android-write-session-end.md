---
nfc: patch
nfc-js: patch
---

Fixed `write` on Android without a kept-alive `scan` session writing the message to every tag tapped afterwards: the session now ends after the first tag is written (or fails to be written), matching iOS.
