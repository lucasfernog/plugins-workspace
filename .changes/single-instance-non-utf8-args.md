---
single-instance: patch
---

Fixed a panic in the second instance when one of its command line arguments is not valid Unicode (common for file paths on Linux). Such arguments, and a non-Unicode working directory, are now forwarded to the first instance with lossy conversion instead of crashing (the working directory used to be replaced by an empty string).
