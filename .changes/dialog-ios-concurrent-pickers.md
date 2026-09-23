---
dialog: patch
dialog-js: patch
---

On iOS, opening a file or save dialog while another dialog is on screen now rejects with an error instead of leaving one of the requests pending forever (which also blocked a thread on the Rust side). A picker that disappeared without reporting a result is settled as cancelled when the next one opens.
