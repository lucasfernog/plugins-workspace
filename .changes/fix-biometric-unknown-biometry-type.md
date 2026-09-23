---
biometric: minor
biometric-js: minor
---

Fixed `Biometric::status` failing to deserialize on devices that report a biometry type the Rust `BiometryType` enum has no variant for (Android iris, Apple Optic ID); those are now reported as `BiometryType::None`. Added `BiometryType.OpticID` to the JavaScript enum.
