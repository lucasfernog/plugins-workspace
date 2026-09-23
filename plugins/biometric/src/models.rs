// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Options for [`Biometric::authenticate`](crate::Biometric::authenticate).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOptions {
    /// Enables authentication using the device's passcode, PIN or pattern, as a fallback for or
    /// instead of biometrics. Available on Android and iOS.
    ///
    /// On iOS this also lets the user authenticate with the passcode when biometry is unavailable.
    pub allow_device_credential: bool,
    /// Label for the cancel button. Available on Android and iOS.
    ///
    /// On Android it is ignored when [`Self::allow_device_credential`] is `true`, because the
    /// system shows its own button to switch to the device credential instead.
    pub cancel_title: Option<String>,
    /// Text displayed on the fallback button shown after a failed biometric attempt. Only
    /// available on iOS.
    ///
    /// When [`Self::allow_device_credential`] is `false`, the button makes the authentication
    /// fail with the `userFallback` error code; an empty string hides it.
    pub fallback_title: Option<String>,
    /// Title indicating the purpose of the biometric verification. Only available on Android.
    pub title: Option<String>,
    /// Subtitle providing contextual information about the biometric verification. Only
    /// available on Android.
    pub subtitle: Option<String>,
    /// Whether additional user confirmation, such as pressing a button, is required after a
    /// successful passive biometric authentication (e.g. face). Only available on Android.
    pub confirmation_required: Option<bool>,
}

/// The kind of biometry hardware detected on the device.
#[derive(Debug, Clone, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum BiometryType {
    /// No supported biometry hardware was detected.
    None = 0,
    /// Fingerprint authentication (Apple Touch ID or Android fingerprint).
    TouchID = 1,
    /// Face authentication (Apple Face ID or Android face authentication).
    FaceID = 2,
}

/// The result of [`Biometric::status`](crate::Biometric::status), describing whether biometric
/// authentication can currently be used.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Whether the device can currently authenticate using biometrics.
    pub is_available: bool,
    /// The kind of biometry hardware detected on the device, even when [`Self::is_available`] is `false`.
    ///
    /// On Android this reflects the device's hardware features, not what the user enrolled: it is
    /// reported even when nothing is enrolled, and when several kinds are present the first of
    /// fingerprint, face and iris is reported.
    pub biometry_type: BiometryType,
    /// A human-readable reason why biometric authentication is unavailable. Only set when
    /// [`Self::is_available`] is `false`.
    pub error: Option<String>,
    /// A platform-specific error code describing why biometric authentication is unavailable.
    /// Only set when [`Self::is_available`] is `false`.
    pub error_code: Option<String>,
}
