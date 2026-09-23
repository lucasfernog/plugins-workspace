// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Perform operations on the current process.
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Exits the app with the given exit code.
 *
 * This emits `RunEvent::ExitRequested` (and then `RunEvent::Exit`) on the Rust side,
 * so the app can still prevent the exit with `api.prevent_exit()`.
 *
 * Requires the `process:allow-exit` permission (included in `process:default`).
 *
 * #### Platform-specific
 *
 * - **iOS:** iOS does not allow an app to quit programmatically: only `ExitRequested` is emitted
 *   and the app keeps running.
 *
 * @example
 * ```typescript
 * import { exit } from '@tauri-apps/plugin-process';
 * await exit(1);
 * ```
 *
 * @param code The exit code to use. Defaults to `0`.
 * @returns A promise that resolves once the exit has been requested. The command cannot fail,
 * and the app may terminate before the promise settles.
 *
 * @since 2.0.0
 */
async function exit(code = 0): Promise<void> {
  await invoke('plugin:process|exit', { code })
}

/**
 * Exits the current instance of the app then relaunches it.
 *
 * This emits `RunEvent::ExitRequested` (with `tauri::RESTART_EXIT_CODE`) and then `RunEvent::Exit`
 * on the Rust side, so the app can still prevent it with `api.prevent_exit()`.
 *
 * Requires the `process:allow-restart` permission (included in `process:default`):
 * the underlying command is named `restart`.
 *
 * #### Platform-specific
 *
 * - **Android:** the app is terminated but not started again.
 * - **iOS:** iOS does not allow an app to quit programmatically: only `ExitRequested` is emitted
 *   and the app keeps running.
 *
 * @example
 * ```typescript
 * import { relaunch } from '@tauri-apps/plugin-process';
 * await relaunch();
 * ```
 *
 * @returns A promise that resolves once the restart has been requested. The command cannot fail,
 * and the app may terminate before the promise settles.
 *
 * @since 2.0.0
 */
async function relaunch(): Promise<void> {
  await invoke('plugin:process|restart')
}

export { exit, relaunch }
