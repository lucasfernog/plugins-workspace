// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// All errors that can occur while running the os plugin.
///
/// This enum currently has no variants: none of the plugin's commands can fail.
#[deprecated(
    since = "2.4.0",
    note = "no API of the plugin returns this error; it will be removed in v3"
)]
#[derive(Debug, thiserror::Error)]
pub enum Error {}

#[allow(deprecated)]
impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
