---
single-instance: patch
---

On Windows, the first instance now validates the `WM_COPYDATA` message it receives from other processes and only reads the number of bytes the sender declared, instead of reading until a NUL terminator. A malformed message (no terminator, or a null pointer) could previously crash the app or read out of bounds.
