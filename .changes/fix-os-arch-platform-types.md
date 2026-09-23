---
os: patch
os-js: patch
---

Added the missing values to the `Arch` type (`loongarch64`, `riscv32`, `sparc`, `m68k`, `csky`, `mips32r6`, `mips64r6`) and `'illumos'` to the `Platform` type. `arch()` and `platform()` already returned them.
