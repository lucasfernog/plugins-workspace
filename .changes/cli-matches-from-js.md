---
cli: minor
cli-js: minor
---

Added `getMatchesFrom(args)` to parse a given list of arguments against the CLI configuration, like the Rust `Cli::matches_from`. It is useful for arguments received after startup, such as the `argv` of a second instance. It requires the new `cli:allow-cli-matches-from` permission, which is not part of `cli:default`.
