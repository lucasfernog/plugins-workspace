// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, Serializer};
use std::fmt::Display;

/// Arguments of the [`Nfc::scan`](crate::Nfc::scan) API.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRequest {
    /// The kind of scan to perform, which defines how tags are matched.
    pub kind: ScanKind,
    /// Whether the connection to the scanned tag must be kept open after the scan resolves.
    ///
    /// When `true`, a following [`Nfc::write`](crate::Nfc::write) call writes to the tag that was
    /// scanned instead of starting a new session.
    pub keep_session_alive: bool,
}

/// Options of the [`Nfc::write_with_options`](crate::Nfc::write_with_options) API.
///
/// Create it with [`WriteOptions::new`] (or [`Default`]) and the builder methods.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WriteOptions {
    /// The kind of scan to perform to find the tag to write to, when there is no
    /// kept-alive [`Nfc::scan`](crate::Nfc::scan) session.
    ///
    /// Required on Android in that case. On iOS a [`ScanKind::Ndef`] scan is performed when it is not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ScanKind>,
    /// Message displayed in the UI while scanning for the tag. **iOS only**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Message displayed in the UI when the tag has been read. **iOS only**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub successful_read_message: Option<String>,
    /// Message displayed in the UI when the message has been written. **iOS only**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_message: Option<String>,
}

impl WriteOptions {
    /// Creates empty write options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the kind of scan to perform to find the tag to write to. See [`Self::kind`].
    pub fn kind(mut self, kind: ScanKind) -> Self {
        self.kind.replace(kind);
        self
    }

    /// Sets the message displayed in the UI while scanning for the tag. **iOS only**.
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message.replace(message.into());
        self
    }

    /// Sets the message displayed in the UI when the tag has been read. **iOS only**.
    pub fn successful_read_message(mut self, message: impl Into<String>) -> Self {
        self.successful_read_message.replace(message.into());
        self
    }

    /// Sets the message displayed in the UI when the message has been written. **iOS only**.
    pub fn success_message(mut self, message: impl Into<String>) -> Self {
        self.success_message.replace(message.into());
        self
    }
}

/// An NDEF record to be written to a tag.
///
/// Use [`NFCTypeNameFormat`] to describe how [`Self::kind`] must be interpreted.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NfcRecord {
    /// The Type Name Format (TNF) of the record.
    pub format: NFCTypeNameFormat,
    /// The record type, interpreted according to [`Self::format`].
    ///
    /// For [`NFCTypeNameFormat::NfcWellKnown`] records this is a Record Type Definition (RTD)
    /// value such as `[0x54]` (`RTD_TEXT`) or `[0x55]` (`RTD_URI`).
    pub kind: Vec<u8>,
    /// The record identifier. Can be empty.
    pub id: Vec<u8>,
    /// The record payload bytes.
    pub payload: Vec<u8>,
}

/// The Type Name Format (TNF) of an NDEF record, which defines how the record type is interpreted.
///
/// Serialized as its numeric value.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde_repr::Deserialize_repr, serde_repr::Serialize_repr,
)]
#[repr(u8)]
pub enum NFCTypeNameFormat {
    /// The record is empty: type, identifier and payload must be empty.
    Empty = 0,
    /// The record type is an NFC Forum well known type, defined by a Record Type Definition (RTD)
    /// such as `RTD_TEXT` (`[0x54]`) or `RTD_URI` (`[0x55]`).
    NfcWellKnown = 1,
    /// The record type is a MIME media type as defined in RFC 2046, e.g. `text/plain`.
    Media = 2,
    /// The record type is an absolute URI as defined in RFC 3986.
    AbsoluteURI = 3,
    /// The record type is an NFC Forum external type, i.e. a type namespaced by its issuer.
    NfcExternal = 4,
    /// The record type is unknown: the type must be empty and the payload interpretation is
    /// left to the application.
    Unknown = 5,
    /// The record is a middle or last chunk of a chunked record and inherits the type of the
    /// first chunk, so its own type must be empty.
    Unchanged = 6,
}

/// An NDEF record read from a scanned tag.
#[derive(Debug, Clone, Deserialize)]
pub struct NfcTagRecord {
    /// The Type Name Format (TNF) of the record, which defines how [`Self::kind`] is interpreted.
    pub tnf: NFCTypeNameFormat,
    /// The record type bytes.
    pub kind: Vec<u8>,
    /// The record identifier bytes. Can be empty.
    pub id: Vec<u8>,
    /// The record payload bytes.
    pub payload: Vec<u8>,
}

/// An NFC tag that has been scanned.
#[derive(Debug, Clone, Deserialize)]
pub struct NfcTag {
    /// The tag identifier, as reported by the operating system.
    pub id: String,
    /// The technology the tag supports.
    pub kind: String,
    /// The NDEF records stored on the tag. Empty when the tag holds no NDEF message.
    pub records: Vec<NfcTagRecord>,
}

/// Response of the [`Nfc::scan`](crate::Nfc::scan) API.
#[derive(Debug, Clone, Deserialize)]
pub struct ScanResponse {
    /// The tag that has been scanned.
    pub tag: NfcTag,
}

/// Filters the tags to scan by the URI of their NDEF payload.
///
/// Every field is optional and only the ones that are set take part in the filter.
/// **Android only**: the iOS implementation ignores this filter.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UriFilter {
    /// Only match URIs with this scheme, e.g. `https`.
    scheme: Option<String>,
    /// Only match URIs with this authority (host), e.g. `tauri.app`.
    host: Option<String>,
    /// Only match URIs whose path starts with this prefix, e.g. `/docs`.
    path_prefix: Option<String>,
}

/// The NFC technologies a tag can support, mirroring the `android.nfc.tech` classes.
///
/// **Android only**. Serialized as the technology name, e.g. `"IsoDep"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TechKind {
    /// ISO-DEP (ISO 14443-4) properties and I/O operations.
    IsoDep,
    /// MIFARE Classic properties and I/O operations.
    MifareClassic,
    /// MIFARE Ultralight and MIFARE Ultralight C properties and I/O operations.
    MifareUltralight,
    /// NDEF data and operations on tags that are already formatted as NDEF.
    Ndef,
    /// Formatting operations on tags that can be formatted as NDEF but are not yet.
    NdefFormatable,
    /// NFC-A (ISO 14443-3A) properties and I/O operations.
    NfcA,
    /// NFC-B (ISO 14443-3B) properties and I/O operations.
    NfcB,
    /// NFC Barcode (Kovio NFC Barcode) properties and I/O operations.
    NfcBarcode,
    /// NFC-F (JIS 6319-4) properties and I/O operations.
    NfcF,
    /// NFC-V (ISO 15693) properties and I/O operations.
    NfcV,
}

impl Display for TechKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IsoDep => "IsoDep",
                Self::MifareClassic => "MifareClassic",
                Self::MifareUltralight => "MifareUltralight",
                Self::Ndef => "Ndef",
                Self::NdefFormatable => "NdefFormatable",
                Self::NfcA => "NfcA",
                Self::NfcB => "NfcB",
                Self::NfcBarcode => "NfcBarcode",
                Self::NfcF => "NfcF",
                Self::NfcV => "NfcV",
            }
        )
    }
}

impl Serialize for TechKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// The kind of scan to perform, which defines which tags are matched.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanKind {
    /// Only match tags that carry an NDEF message.
    Ndef {
        /// Only match tags whose NDEF payload has this MIME type, e.g. `text/plain`.
        /// **Android only**.
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        /// Only match tags whose NDEF payload URI matches this filter. **Android only**.
        uri: Option<UriFilter>,
        /// Only match tags supporting the listed technologies.
        ///
        /// Each tech list is considered independently and the tag matches when any single tech
        /// list matches it, which provides AND (inside a list) and OR (between lists) semantics.
        ///
        /// **Android only**. See
        /// <https://developer.android.com/reference/android/nfc/NfcAdapter#ACTION_TECH_DISCOVERED>
        /// for more information.
        #[serde(rename = "techLists")]
        tech_list: Option<Vec<Vec<TechKind>>>,
    },
    /// Match any tag that is discovered, whether it carries an NDEF message or not.
    Tag {
        /// Only match tags whose payload has this MIME type, e.g. `text/plain`. **Android only**.
        #[serde(rename = "mimeType")]
        mime_type: Option<String>,
        /// Only match tags whose payload URI matches this filter. **Android only**.
        uri: Option<UriFilter>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_kind_fields_are_serialized_as_the_native_side_expects() {
        let ndef = ScanKind::Ndef {
            mime_type: Some("text/plain".into()),
            uri: Some(UriFilter {
                scheme: Some("https".into()),
                host: Some("tauri.app".into()),
                path_prefix: Some("/docs".into()),
            }),
            tech_list: Some(vec![vec![TechKind::NfcA, TechKind::Ndef]]),
        };
        assert_eq!(
            serde_json::to_value(&ndef).unwrap(),
            serde_json::json!({
                "ndef": {
                    "mimeType": "text/plain",
                    "uri": { "scheme": "https", "host": "tauri.app", "pathPrefix": "/docs" },
                    "techLists": [["NfcA", "Ndef"]]
                }
            })
        );

        let tag = ScanKind::Tag {
            mime_type: Some("text/plain".into()),
            uri: None,
        };
        assert_eq!(
            serde_json::to_value(&tag).unwrap(),
            serde_json::json!({ "tag": { "mimeType": "text/plain", "uri": null } })
        );
    }

    #[test]
    fn write_options_are_serialized_as_the_native_side_expects() {
        assert_eq!(
            serde_json::to_value(WriteOptions::new()).unwrap(),
            serde_json::json!({})
        );
        let options = WriteOptions::new()
            .kind(ScanKind::Tag {
                mime_type: None,
                uri: None,
            })
            .message("Hold your device near the tag")
            .successful_read_message("Tag found")
            .success_message("Tag written");
        assert_eq!(
            serde_json::to_value(options).unwrap(),
            serde_json::json!({
                "kind": { "tag": { "mimeType": null, "uri": null } },
                "message": "Hold your device near the tag",
                "successfulReadMessage": "Tag found",
                "successMessage": "Tag written"
            })
        );
    }
}
