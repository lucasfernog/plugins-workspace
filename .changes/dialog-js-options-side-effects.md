---
dialog: patch
dialog-js: patch
---

`message` no longer writes a `buttons` property onto the options object it is given (which threw a `TypeError` for frozen objects), and `open` and `save` no longer freeze the options object passed to them.
