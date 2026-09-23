// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Open files and URLs using their default application, and reveal files in the system's file explorer.
 *
 * ## Security
 *
 * {@link openUrl} and {@link openPath} are restricted by the scope of the capability that allows them.
 * Each scope entry is either a `url` or a `path` glob pattern, plus an optional `app` field that controls
 * which program may be passed as `openWith`:
 *
 * - omitted: only the default application may be used (`openWith` must not be set),
 * - `true`: any application may be used,
 * - `false`: the entry never matches,
 * - `"<name>"`: only this specific application may be used.
 *
 * ```json
 * {
 *   "permissions": [
 *     "opener:default",
 *     {
 *       "identifier": "opener:allow-open-path",
 *       "allow": [{ "path": "$APPDATA/**" }]
 *     }
 *   ]
 * }
 * ```
 *
 * `opener:default` allows opening `http(s)://`, `mailto:` and `tel:` URLs with their default application,
 * and {@link revealItemInDir} for any path. {@link openPath} is not allowed by default.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Opens a url with the system's default app, or the one specified with {@linkcode openWith}.
 *
 * @example
 * ```typescript
 * import { openUrl } from '@tauri-apps/plugin-opener';
 *
 * // opens the given URL on the default browser:
 * await openUrl('https://github.com/tauri-apps/tauri');
 * // opens the given URL using `firefox`:
 * await openUrl('https://github.com/tauri-apps/tauri', 'firefox');
 * ```
 *
 * @param url The URL to open.
 * @param openWith The app to open the URL with. If not specified, defaults to the system default application for the specified url type.
 * It must be allowed by the `app` field of the matching scope entry.
 * On mobile, `openWith` can be provided as `inAppBrowser` to open the URL in an in-app browser. Otherwise, it will open the URL in the system default browser.
 *
 * @since 2.0.0
 */
export async function openUrl(
  url: string | URL,
  // eslint-disable-next-line @typescript-eslint/no-redundant-type-constituents
  openWith?: 'inAppBrowser' | string
): Promise<void> {
  await invoke('plugin:opener|open_url', {
    url,
    with: openWith
  })
}

/**
 * Opens a path with the system's default app, or the one specified with {@linkcode openWith}.
 *
 * @example
 * ```typescript
 * import { openPath } from '@tauri-apps/plugin-opener';
 *
 * // opens a file using the default program:
 * await openPath('/path/to/file');
 * // opens a file using `vlc` (requires a scope entry with `"app": "vlc"`):
 * await openPath('/path/to/file', 'vlc');
 * ```
 *
 * @param path The path to open.
 * @param openWith The app to open the path with. If not specified, defaults to the system default application for the specified path type.
 * On Linux and Windows this is a program name or path (e.g. `vlc`), on macOS an application name (e.g. `VLC` or `Visual Studio Code`).
 * It must be allowed by the `app` field of the matching scope entry. Opening an executable file with the default program runs it.
 *
 * @since 2.0.0
 */
export async function openPath(path: string, openWith?: string): Promise<void> {
  await invoke('plugin:opener|open_path', {
    path,
    with: openWith
  })
}

/**
 * Reveal a path with the system's default explorer.
 *
 * #### Platform-specific:
 *
 * - **Android / iOS:** Unsupported.
 *
 * @example
 * ```typescript
 * import { revealItemInDir } from '@tauri-apps/plugin-opener';
 * await revealItemInDir('/path/to/file');
 * await revealItemInDir([ '/path/to/file', '/path/to/another/file' ]);
 * ```
 *
 * @param path The path to reveal, or an array of paths (since 2.5.0). The paths must exist.
 * On Linux, if the file manager does not support revealing items, only the directory of the first path is opened.
 *
 * @since 2.0.0
 */
export async function revealItemInDir(path: string | string[]): Promise<void> {
  const paths = typeof path === 'string' ? [path] : path
  return invoke('plugin:opener|reveal_item_in_dir', { paths })
}
