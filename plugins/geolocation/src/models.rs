// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::plugin::PermissionState;

/// The current permission state for the geolocation APIs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    /// Permission state for the location alias.
    ///
    /// On Android it requests/checks both ACCESS_COARSE_LOCATION and ACCESS_FINE_LOCATION permissions.
    ///
    /// On iOS it requests/checks location permissions.
    pub location: PermissionState,
    /// Permissions state for the coarseLoaction alias.
    ///
    /// On Android it requests/checks ACCESS_COARSE_LOCATION.
    ///
    /// On Android 12+, users can choose between Approximate location (ACCESS_COARSE_LOCATION) and Precise location (ACCESS_FINE_LOCATION).
    ///
    /// On iOS it will have the same value as the `location` alias.
    pub coarse_location: PermissionState,
}

/// Options used to configure a [`get_current_position`](crate::Geolocation::get_current_position) or [`watch_position`](crate::Geolocation::watch_position) request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct PositionOptions {
    /// High accuracy mode (such as GPS, if available)
    /// Will be ignored on Android 12+ if users didn't grant the ACCESS_FINE_LOCATION permission.
    ///
    /// Defaults to `false` when missing from the deserialized options.
    #[serde(default)]
    pub enable_high_accuracy: bool,
    /// The maximum wait time in milliseconds for location updates.
    /// On Android the timeout gets ignored for getCurrentPosition.
    /// Ignored on iOS.
    ///
    /// Defaults to `10000` when missing or `null` (what JavaScript's `Infinity` serializes to) in the deserialized options.
    /// Note that [`PositionOptions::default`] sets it to `0`.
    // TODO: Should be u64+ but specta doesn't like that?
    #[serde(default = "default_timeout", deserialize_with = "deserialize_timeout")]
    pub timeout: u32,
    /// The maximum age in milliseconds of a possible cached position that is acceptable to return.
    /// Ignored on iOS.
    ///
    /// Defaults to `0` when missing from the deserialized options. `null` (what JavaScript's `Infinity`
    /// serializes to) accepts a cached position of any age.
    // TODO: Should be u64+ but specta doesn't like that?
    #[serde(default, deserialize_with = "deserialize_maximum_age")]
    pub maximum_age: u32,
}

const DEFAULT_TIMEOUT: u32 = 10000;

fn default_timeout() -> u32 {
    DEFAULT_TIMEOUT
}

/// Converts a JavaScript number to milliseconds, clamping it to the `u32` range.
/// `null` (a non-finite number such as `Infinity`) maps to `if_null`, `NaN` to `if_nan`.
fn millis_from_js_number(value: Option<f64>, if_null: u32, if_nan: u32) -> u32 {
    match value {
        None => if_null,
        Some(value) if value.is_nan() => if_nan,
        // `as` saturates: negative values become 0 and values past u32::MAX (or infinite) become u32::MAX.
        Some(value) => value as u32,
    }
}

fn deserialize_timeout<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<u32, D::Error> {
    Option::<f64>::deserialize(deserializer)
        .map(|value| millis_from_js_number(value, DEFAULT_TIMEOUT, DEFAULT_TIMEOUT))
}

fn deserialize_maximum_age<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<u32, D::Error> {
    Option::<f64>::deserialize(deserializer).map(|value| millis_from_js_number(value, u32::MAX, 0))
}

/// The individual permission aliases that can be requested with [`request_permissions`](crate::Geolocation::request_permissions).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum PermissionType {
    /// The `location` alias. On Android this maps to both `ACCESS_COARSE_LOCATION` and `ACCESS_FINE_LOCATION`. On iOS it maps to the standard location permission.
    Location,
    /// The `coarseLocation` alias. On Android this maps to `ACCESS_COARSE_LOCATION` only. On iOS it behaves the same as [`Location`](Self::Location).
    CoarseLocation,
}

/// The GPS coordinates of a [`Position`], along with the accuracy of each reading.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Coordinates {
    /// Latitude in decimal degrees.
    pub latitude: f64,
    /// Longitude in decimal degrees.
    pub longitude: f64,
    /// Accuracy level of the latitude and longitude coordinates in meters.
    pub accuracy: f64,
    /// Accuracy level of the altitude coordinate in meters, if available.
    /// Available on all iOS versions and on Android 8 and above.
    pub altitude_accuracy: Option<f64>,
    /// The altitude the user is at, if available.
    pub altitude: Option<f64>,
    /// The speed the user is traveling, in meters per second, if available.
    pub speed: Option<f64>,
    /// The heading the user is facing, if available.
    pub heading: Option<f64>,
}

/// A geolocation reading, as returned by [`get_current_position`](crate::Geolocation::get_current_position) and reported through [`WatchEvent::Position`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Creation time for these coordinates.
    // TODO: Check if we're actually losing precision.
    pub timestamp: u64,
    /// The GPS coordinates along with the accuracy of the data.
    pub coords: Coordinates,
}

/// A single update sent through the channel callback registered with [`watch_position`](crate::Geolocation::watch_position).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(untagged)]
pub enum WatchEvent {
    /// A new position was read successfully.
    Position(Position),
    /// The platform failed to read a position; the string is the platform-provided error message.
    Error(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(json: serde_json::Value) -> PositionOptions {
        serde_json::from_value(json).expect("valid options")
    }

    #[test]
    fn position_options_full() {
        let o = options(serde_json::json!({
            "enableHighAccuracy": true,
            "timeout": 5000,
            "maximumAge": 1000
        }));
        assert!(o.enable_high_accuracy);
        assert_eq!(o.timeout, 5000);
        assert_eq!(o.maximum_age, 1000);
    }

    #[test]
    fn position_options_partial_uses_defaults() {
        let o = options(serde_json::json!({ "enableHighAccuracy": true }));
        assert!(o.enable_high_accuracy);
        assert_eq!(o.timeout, DEFAULT_TIMEOUT);
        assert_eq!(o.maximum_age, 0);

        let o = options(serde_json::json!({}));
        assert!(!o.enable_high_accuracy);
        assert_eq!(o.timeout, DEFAULT_TIMEOUT);
        assert_eq!(o.maximum_age, 0);
    }

    #[test]
    fn position_options_infinity_and_out_of_range() {
        // `JSON.stringify(Infinity)` is `null`.
        let o = options(serde_json::json!({ "timeout": null, "maximumAge": null }));
        assert_eq!(o.timeout, DEFAULT_TIMEOUT);
        assert_eq!(o.maximum_age, u32::MAX);

        let o = options(serde_json::json!({ "timeout": -1, "maximumAge": 1e12 }));
        assert_eq!(o.timeout, 0);
        assert_eq!(o.maximum_age, u32::MAX);

        let o = options(serde_json::json!({ "timeout": 1500.7, "maximumAge": 0.2 }));
        assert_eq!(o.timeout, 1500);
        assert_eq!(o.maximum_age, 0);
    }

    #[test]
    fn position_options_rejects_non_numbers() {
        assert!(
            serde_json::from_value::<PositionOptions>(serde_json::json!({ "timeout": "1" }))
                .is_err()
        );
    }

    #[test]
    fn position_options_round_trip() {
        let o = PositionOptions {
            enable_high_accuracy: true,
            timeout: 42,
            maximum_age: 7,
        };
        let o: PositionOptions = serde_json::from_value(serde_json::to_value(&o).unwrap()).unwrap();
        assert!(o.enable_high_accuracy);
        assert_eq!(o.timeout, 42);
        assert_eq!(o.maximum_age, 7);
    }
}
