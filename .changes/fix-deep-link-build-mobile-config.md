---
deep-link: patch
deep-link-js: patch
---

Fixed the build with some `mobile` configurations: hosts, schemes and path values are now XML-escaped in the generated `AndroidManifest.xml` intent filters (a `pathPattern` containing `&` or `"` produced a broken manifest), and a non app link entry without custom schemes (an empty `scheme` list, or only `http`/`https`) no longer panics the build on macOS or adds an empty `CFBundleURLTypes` entry to the iOS `Info.plist`.
