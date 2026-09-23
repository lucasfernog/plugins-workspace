// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Save and restore window positions and sizes.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { type WindowLabel, getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Flags controlling which parts of a window's state are saved and restored.
 * Combine multiple flags with the bitwise OR operator (`|`).
 */
export enum StateFlags {
  /** Save and restore the window size. */
  SIZE = 1 << 0,
  /** Save and restore the window position. */
  POSITION = 1 << 1,
  /** Save and restore whether the window is maximized. */
  MAXIMIZED = 1 << 2,
  /** Save and restore whether the window is visible. */
  VISIBLE = 1 << 3,
  /** Save and restore whether the window has decorations. */
  DECORATIONS = 1 << 4,
  /** Save and restore whether the window is fullscreen. */
  FULLSCREEN = 1 << 5,
  /** Save and restore every flag above. */
  ALL = SIZE | POSITION | MAXIMIZED | VISIBLE | DECORATIONS | FULLSCREEN
}

/**
 *  Save the state of all open windows to disk.
 *
 * @example
 * ```typescript
 * import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state';
 *
 * await saveWindowState(StateFlags.ALL);
 * ```
 *
 * @param flags Which parts of the state to save. Defaults to the flags passed to the plugin `Builder` (all flags if none were set).
 * @since 2.0.0
 */
async function saveWindowState(flags?: StateFlags): Promise<void> {
  await invoke('plugin:window-state|save_window_state', { flags })
}

/**
 *  Restore the state for the specified window.
 *
 * The state comes from the plugin's in-memory cache, which is loaded from the
 * state file when the app starts and kept up to date while it runs; the file is
 * not read again. If nothing is cached for the window yet, its current state is
 * cached instead. With {@linkcode StateFlags.VISIBLE}, the window is also shown
 * and focused, unless it was saved as hidden.
 *
 * Rejects if there is no window with the given label.
 *
 * @example
 * ```typescript
 * import { restoreState, StateFlags } from '@tauri-apps/plugin-window-state';
 *
 * await restoreState('main', StateFlags.ALL);
 * ```
 *
 * @param label The label of the window to restore.
 * @param flags Which parts of the state to restore. Defaults to the flags passed to the plugin `Builder` (all flags if none were set).
 * @since 2.0.0
 */
async function restoreState(
  label: WindowLabel,
  flags?: StateFlags
): Promise<void> {
  await invoke('plugin:window-state|restore_state', { label, flags })
}

/**
 *  Restore the state for the current window. See {@linkcode restoreState}.
 *
 * @example
 * ```typescript
 * import { restoreStateCurrent, StateFlags } from '@tauri-apps/plugin-window-state';
 *
 * await restoreStateCurrent(StateFlags.ALL);
 * ```
 *
 * @param flags Which parts of the state to restore. Defaults to the flags passed to the plugin `Builder` (all flags if none were set).
 * @since 2.0.0
 */
async function restoreStateCurrent(flags?: StateFlags): Promise<void> {
  await restoreState(getCurrentWindow().label, flags)
}
/**
 *  Get the name of the file used to store window state.
 *
 * @example
 * ```typescript
 * import { filename } from '@tauri-apps/plugin-window-state';
 *
 * const name = await filename();
 * ```
 *
 * @returns A promise resolving to the name of the file used to store window state.
 * @since 2.0.0
 */
async function filename(): Promise<string> {
  return await invoke('plugin:window-state|filename')
}

export { restoreState, restoreStateCurrent, saveWindowState, filename }
