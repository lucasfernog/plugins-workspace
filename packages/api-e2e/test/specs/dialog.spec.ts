// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, itOn } from '../helpers/index.js'

// Every dialog blocks on native UI the driver cannot operate, so nothing here
// opens one. The API surface itself is asserted in `plugins.spec.ts`.

describePlugin('dialog', () => {
  // The plugin injects `window.alert`/`window.confirm` overrides everywhere but
  // Android. Calling them would open a dialog, so check which command they use.
  itOn(
    ['linux', 'win32', 'darwin', 'ios'],
    'overrides window.alert and window.confirm with the message command',
    async () => {
      // the functions are only stringified, never called
      /* eslint-disable @typescript-eslint/unbound-method */
      const sources = await tauri(() => ({
        alert: String(window.alert),
        confirm: String(window.confirm)
      }))
      /* eslint-enable @typescript-eslint/unbound-method */
      expect(sources.alert).toContain('plugin:dialog|message')
      expect(sources.confirm).toContain('plugin:dialog|message')
      expect(sources.confirm).toContain('OkCancel')
      // There is no `confirm` command, so the override used to always reject.
      expect(sources.confirm).not.toContain('plugin:dialog|confirm')
    }
  )
})
