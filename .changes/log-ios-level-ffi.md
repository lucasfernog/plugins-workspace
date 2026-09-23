---
log: patch
log-js: patch
---

Fixed the iOS `tauri_log` Swift function declaring its `level` parameter as a 64-bit `Int` while Rust passes a `u8`, which could make records be dropped if the upper bits of the register were not zeroed.
