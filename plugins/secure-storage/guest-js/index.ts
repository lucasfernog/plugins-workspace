// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Store secrets such as tokens and passwords in the operating system's secure storage:
 * the Keychain on macOS and iOS, the Credential Manager on Windows, the Secret Service
 * on Linux and the Keystore on Android.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Reads the value stored under `key`.
 *
 * On Android this also resolves to `null` when the stored value can no longer be
 * decrypted, which happens when the app data was restored from a backup.
 *
 * @example
 * ```typescript
 * import { getItem } from '@tauri-apps/plugin-secure-storage';
 * const token = await getItem('access-token');
 * if (token === null) {
 *   // not stored yet
 * }
 * ```
 *
 * @param key The name of the item. Must not be empty.
 * @returns A promise resolving to the stored value, or `null` if there is none.
 *
 * @since 2.0.0
 */
async function getItem(key: string): Promise<string | null> {
  return await invoke('plugin:secure-storage|get_item', { key })
}

/**
 * Stores `value` under `key`, replacing any existing value.
 *
 * Keep values small: the Windows Credential Manager rejects values larger than 2560 bytes.
 *
 * @example
 * ```typescript
 * import { setItem } from '@tauri-apps/plugin-secure-storage';
 * await setItem('access-token', 'eyJhbGciOi...');
 * ```
 *
 * @param key The name of the item. Must not be empty.
 * @param value The secret to store.
 *
 * @since 2.0.0
 */
async function setItem(key: string, value: string): Promise<void> {
  await invoke('plugin:secure-storage|set_item', { key, value })
}

/**
 * Removes the value stored under `key`. Does nothing if there is none.
 *
 * @example
 * ```typescript
 * import { removeItem } from '@tauri-apps/plugin-secure-storage';
 * await removeItem('access-token');
 * ```
 *
 * @param key The name of the item. Must not be empty.
 *
 * @since 2.0.0
 */
async function removeItem(key: string): Promise<void> {
  await invoke('plugin:secure-storage|remove_item', { key })
}

export { getItem, setItem, removeItem }
