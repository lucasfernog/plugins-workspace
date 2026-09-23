// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  platform
} from '../helpers/index.js'

// Triggering a shortcut needs OS-level synthetic input the WebDriver session
// cannot produce, so the specs cover the registry (register / isRegistered /
// unregister / unregisterAll) and the error paths. The plugin is desktop-only,
// so the whole suite is skipped on mobile.

describePlugin('global-shortcut', { desktopOnly: true }, () => {
  afterEach(async () => {
    await tauri((api) => api.globalShortcut.unregisterAll())
  })

  it('register and unregister update isRegistered', async () => {
    const result = await tauri(async (api) => {
      const shortcut = 'CommandOrControl+Shift+F9'
      const before = await api.globalShortcut.isRegistered(shortcut)
      await api.globalShortcut.register(shortcut, () => {})
      const registered = await api.globalShortcut.isRegistered(shortcut)
      await api.globalShortcut.unregister(shortcut)
      const after = await api.globalShortcut.isRegistered(shortcut)
      return { before, registered, after }
    })
    expect(result).toEqual({ before: false, registered: true, after: false })
  })

  it('register accepts a list of shortcuts and unregister a list too', async () => {
    const result = await tauri(async (api) => {
      const shortcuts = ['Alt+Shift+F7', 'Alt+Shift+F8']
      await api.globalShortcut.register(shortcuts, () => {})
      const registered = await Promise.all(
        shortcuts.map((s) => api.globalShortcut.isRegistered(s))
      )
      await api.globalShortcut.unregister(shortcuts)
      const after = await Promise.all(
        shortcuts.map((s) => api.globalShortcut.isRegistered(s))
      )
      return { registered, after }
    })
    expect(result.registered).toEqual([true, true])
    expect(result.after).toEqual([false, false])
  })

  it('unregisterAll clears every registration', async () => {
    const result = await tauri(async (api) => {
      await api.globalShortcut.register('Alt+Shift+F5', () => {})
      await api.globalShortcut.register('Alt+Shift+F6', () => {})
      await api.globalShortcut.unregisterAll()
      return [
        await api.globalShortcut.isRegistered('Alt+Shift+F5'),
        await api.globalShortcut.isRegistered('Alt+Shift+F6')
      ]
    })
    expect(result).toEqual([false, false])
  })

  it('shortcut names are normalized when checking registrations', async () => {
    const result = await tauri(async (api) => {
      await api.globalShortcut.register('CmdOrCtrl+Alt+F10', () => {})
      return {
        aliased: await api.globalShortcut.isRegistered(
          'CommandOrControl+Alt+F10'
        ),
        reordered: await api.globalShortcut.isRegistered('Alt+CmdOrCtrl+F10')
      }
    })
    expect(result).toEqual({ aliased: true, reordered: true })
  })

  it('registering the same shortcut twice rejects', async () => {
    const message = await tauriError(async (api) => {
      await api.globalShortcut.register('Alt+Shift+F11', () => {})
      try {
        await api.globalShortcut.register('Alt+Shift+F11', () => {})
      } finally {
        await api.globalShortcut.unregister('Alt+Shift+F11')
      }
    })
    // macOS does not check for duplicates itself: the OS rejects the second
    // `RegisterEventHotKey` call and the error is the generic one
    expect(message).toMatch(
      platform === 'darwin'
        ? /RegisterEventHotKey failed for F11/
        : /already registered/i
    )
  })

  it('registering a list is all-or-nothing', async () => {
    const result = await tauri(async (api) => {
      const taken = 'CommandOrControl+Shift+F8'
      const fresh = 'CommandOrControl+Shift+F7'
      await api.globalShortcut.register(taken, () => {})
      let rejected = false
      try {
        await api.globalShortcut.register([fresh, taken], () => {})
      } catch {
        rejected = true
      }
      return {
        rejected,
        fresh: await api.globalShortcut.isRegistered(fresh),
        taken: await api.globalShortcut.isRegistered(taken),
        // the rolled back shortcut can be registered again
        retry: await api.globalShortcut
          .register(fresh, () => {})
          .then(() => true)
      }
    })
    expect(result).toEqual({
      rejected: true,
      fresh: false,
      taken: true,
      retry: true
    })
  })

  it('rejects shortcuts that cannot be parsed', async () => {
    const message = await tauriError((api) =>
      api.globalShortcut.register('NotAKey+Nope', () => {})
    )
    expect(message).toMatch(/NotAKey/)
  })
})
