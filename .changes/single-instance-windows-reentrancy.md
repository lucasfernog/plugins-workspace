---
single-instance: patch
---

On Windows, the callback is no longer re-entered when it runs a modal loop (for example a blocking message dialog) and another instance starts meanwhile; the new arguments are queued and handled after the callback returns. The second instance also stops waiting for the first instance after 10 seconds (or right away if it is hung) instead of blocking forever.
