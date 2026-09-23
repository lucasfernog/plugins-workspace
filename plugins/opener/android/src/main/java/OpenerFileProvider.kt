// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.opener

import androidx.core.content.FileProvider

/**
 * A dedicated [FileProvider] subclass, so its manifest entry does not clash with a
 * `androidx.core.content.FileProvider` the app (or another library) declares itself.
 */
class OpenerFileProvider : FileProvider()
