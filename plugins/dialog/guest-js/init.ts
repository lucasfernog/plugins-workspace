// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Injected script that replaces `window.alert` and `window.confirm` with implementations backed
 * by native dialogs. Both use the `message` command, so they require the `dialog:allow-message`
 * permission (part of `dialog:default`).
 *
 * Tauri has no synchronous IPC, so neither override can block the page like the browser
 * built-ins do:
 * - `alert` returns immediately, before the dialog is closed.
 * - `confirm` returns a `Promise<boolean>` that resolves to `true` if the user pressed `Ok`.
 *   It must be awaited: the returned Promise itself is always truthy.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

// like the built-ins, stringify whatever is passed in
function toMessage(message: unknown): string {
  // eslint-disable-next-line @typescript-eslint/no-base-to-string
  return message === undefined ? '' : String(message)
}

window.alert = function (message?: unknown) {
  void invoke('plugin:dialog|message', {
    message: toMessage(message)
  })
}

// @ts-expect-error tauri does not have sync IPC, so this returns a Promise<boolean> instead of a boolean
window.confirm = async function (message?: unknown): Promise<boolean> {
  return (
    (await invoke('plugin:dialog|message', {
      message: toMessage(message),
      buttons: 'OkCancel'
    })) === 'Ok'
  )
}
