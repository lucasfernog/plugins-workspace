---
"haptics": patch
"haptics-js": patch
---

On iOS, `vibrate` no longer leaks a Core Haptics engine on every call. The plugin now keeps a single engine, and a new `vibrate` call replaces the vibration that is still playing, as on Android.
