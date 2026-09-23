---
localhost: minor
---

Added `LocalhostExt::localhost_addr` to read the address the localhost server is bound to. Create the `Builder` with port `0` to let the operating system pick a free port, then build the window URL from this address, instead of picking a port up front (e.g. with `portpicker`) and racing other processes for it.
