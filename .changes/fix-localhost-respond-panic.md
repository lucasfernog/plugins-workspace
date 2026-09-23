---
localhost: patch
---

The localhost server no longer stops serving for the rest of the session when writing a response fails (for example when a client closes the connection early). The error is now logged instead of panicking the server thread.
