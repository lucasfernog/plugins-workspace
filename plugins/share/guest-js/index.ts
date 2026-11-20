// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Share text, links and files through the system share sheet.
 *
 * ## Security
 *
 * Text and URLs can be shared freely. Files can only be shared if the plugin scope allows them,
 * which is configured on the `share:allow-share` permission:
 *
 * ```json
 * {
 *   "identifier": "share:allow-share",
 *   "allow": [{ "path": "$APPDATA/**" }]
 * }
 * ```
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * The content to share. At least one of `text`, `url` and `files` must be set.
 *
 * @since 2.0.0
 */
export interface ShareData {
  /**
   * A title for the shared content.
   *
   * #### Platform-specific
   *
   * - **Android:** Shown at the top of the share sheet and used as the subject by apps that have one, such as email clients.
   * - **iOS:** Used as the subject by apps that have one.
   * - **Windows:** Shown in the share dialog. Defaults to the app name.
   * - **macOS:** Ignored.
   */
  title?: string
  /** The text to share. */
  text?: string
  /** The URL to share. */
  url?: string
  /**
   * The files to share, as file system paths. They must be allowed by the plugin scope.
   *
   * #### Platform-specific
   *
   * - **Android:** The files must be inside one of the app's directories:
   *   the data and cache directories and their external storage counterparts.
   */
  files?: string[]
}

/**
 * Options for how the share sheet is presented.
 *
 * @since 2.0.0
 */
export interface ShareOptions {
  /**
   * The area the share sheet is anchored to, usually the button that triggered it,
   * in CSS pixels relative to the top-left corner of the webview.
   * The result of `element.getBoundingClientRect()` can be used as is.
   *
   * Used on macOS and on iPad, where the share sheet is a popover.
   * Defaults to the center of the window. Ignored on the other platforms.
   */
  anchor?: {
    x: number
    y: number
    width?: number
    height?: number
  }
}

/**
 * Opens the system share sheet for the given content.
 *
 * The promise resolves once the share sheet is shown. Whether the user completed the share
 * or dismissed the sheet is not reported, because most platforms do not tell.
 *
 * It rejects if `data` has no `text`, `url` or `files`, if a file is not allowed by the scope
 * or cannot be read, and on Linux, which has no system share sheet.
 *
 * @example
 * ```typescript
 * import { share } from '@tauri-apps/plugin-share'
 *
 * await share({ title: 'Tauri', text: 'Check this out', url: 'https://tauri.app' })
 * ```
 *
 * @example
 * ```typescript
 * import { share } from '@tauri-apps/plugin-share'
 * import { appDataDir, join } from '@tauri-apps/api/path'
 *
 * // anchor the share sheet to the button that was clicked
 * button.addEventListener('click', async () => {
 *   const report = await join(await appDataDir(), 'report.pdf')
 *   await share({ files: [report] }, { anchor: button.getBoundingClientRect() })
 * })
 * ```
 *
 * @since 2.0.0
 */
export async function share(
  data: ShareData,
  options?: ShareOptions
): Promise<void> {
  // only send the fields the plugin reads, a DOMRect has many more
  const anchor = options?.anchor && {
    x: options.anchor.x,
    y: options.anchor.y,
    width: options.anchor.width ?? 0,
    height: options.anchor.height ?? 0
  }
  await invoke('plugin:share|share', { data, options: { anchor } })
}
