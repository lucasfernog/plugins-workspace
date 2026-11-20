// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.share

import androidx.core.content.FileProvider

// A subclass of its own so the manifest entry does not clash with the app's (or another
// library's) androidx FileProvider declaration.
class ShareFileProvider : FileProvider()
