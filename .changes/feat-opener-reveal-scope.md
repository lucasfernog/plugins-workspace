---
opener: minor
opener-js: minor
---

`revealItemInDir` can now be restricted with a scope: when the `opener:allow-reveal-item-in-dir` permission is given `path` entries (e.g. `{ "identifier": "opener:allow-reveal-item-in-dir", "allow": [{ "path": "$APPDATA/**" }] }`), every revealed path must match them and not match a `deny` entry, otherwise the command fails with the new `Error::ForbiddenRevealPath`. Without such entries the command stays unscoped, as before.
