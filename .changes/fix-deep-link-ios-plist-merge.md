---
deep-link: patch
deep-link-js: patch
---

On iOS, the build no longer wipes associated domains and URL types it does not own: only the `applinks:` entries of the `com.apple.developer.associated-domains` entitlement and the `CFBundleURLTypes` entries the plugin generates are replaced, so `webcredentials:` domains and URL schemes added for other SDKs (OAuth, Google Sign-In, Facebook, ...) are kept.
