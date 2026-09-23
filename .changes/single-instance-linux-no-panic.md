---
single-instance: patch
---

On Linux, the plugin no longer panics when no D-Bus session bus is available (headless sessions, some containers and sandboxes) or when the D-Bus name derived from the identifier is invalid (for example a pre-release version with build metadata such as `1.0.0-beta.1+abc` combined with the `semver` feature). Invalid characters are now replaced with `_`, valid names are unchanged, and other D-Bus errors are logged and the app launches without single-instance protection instead of silently. Failures to notify the running instance are logged too.
