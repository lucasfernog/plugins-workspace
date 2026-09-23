// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin, itOn } from '../helpers/index.js'

// The suite launches the app without a URL, so there is no current deep link,
// and it cannot open one through the OS either. `onOpenUrl` is exercised by
// emitting the event the plugin itself emits when the app is opened with one.
//
// Registering a scheme at runtime is only implemented on Linux (a `.desktop`
// handler plus `xdg-mime`) and Windows (the registry); macOS and mobile read
// the schemes from the bundle and report the runtime APIs as unsupported.

/** A scheme no real app handles, so registering it cannot clobber anything. */
const scheme = 'tauri-plugins-e2e'

describePlugin('deep-link', () => {
  it('getCurrent is null when the app was not opened with a URL', async () => {
    expect(await tauri((api) => api.deepLink.getCurrent())).toBeNull()
  })

  it('onOpenUrl delivers the opened URLs until unlistened', async () => {
    const received = await tauri(
      async (api, urls) => {
        const received: string[][] = []
        const unlisten = await api.deepLink.onOpenUrl((urls) =>
          received.push(urls)
        )
        await api.event.emit('deep-link://new-url', urls)
        await new Promise((resolve) => setTimeout(resolve, 500))
        unlisten()
        await api.event.emit('deep-link://new-url', ['ignored://'])
        await new Promise((resolve) => setTimeout(resolve, 500))
        return received
      },
      [`${scheme}://open/path?query=1`]
    )
    expect(received).toEqual([[`${scheme}://open/path?query=1`]])
  })

  itOn(
    ['linux', 'win32'],
    'register and unregister toggle the scheme handler',
    async () => {
      const result = await tauri(async (api, scheme) => {
        await api.deepLink.register(scheme)
        const registered = await api.deepLink.isRegistered(scheme)
        await api.deepLink.unregister(scheme)
        return {
          registered,
          afterUnregister: await api.deepLink.isRegistered(scheme)
        }
      }, scheme)
      expect(result).toEqual({ registered: true, afterUnregister: false })
    }
  )

  itOn(
    ['linux', 'win32'],
    'isRegistered is false for an unknown scheme',
    async () => {
      expect(
        await tauri(
          (api, scheme) => api.deepLink.isRegistered(scheme),
          `${scheme}-unknown`
        )
      ).toBe(false)
    }
  )

  itOn(
    ['linux', 'win32'],
    'register and unregister reject invalid scheme names',
    async () => {
      for (const command of ['register', 'unregister'] as const) {
        for (const invalid of ['.txt', '*', 'bad\nExec=true', 'a\\b', '']) {
          const error = await tauriError(
            // eslint-disable-next-line security/detect-object-injection
            (api, command, invalid) => api.deepLink[command](invalid),
            command,
            invalid
          )
          expect(error).toMatch(/invalid protocol scheme/)
        }
      }
      expect(
        await tauri((api) => api.deepLink.isRegistered('bad\nscheme'))
      ).toBe(false)
    }
  )

  itOn(
    ['win32'],
    'register refuses to take over a registry class that is not a URL protocol',
    async () => {
      // `txtfile` is the ProgID of `.txt` files on every Windows install
      expect(
        await tauriError((api) => api.deepLink.register('txtfile'))
      ).toMatch(/non URL protocol/)
    }
  )

  itOn(
    ['darwin', 'android', 'ios'],
    'runtime registration is unsupported',
    async () => {
      for (const command of [
        'register',
        'unregister',
        'isRegistered'
      ] as const) {
        const error = await tauriError(
          // eslint-disable-next-line security/detect-object-injection
          (api, command, scheme) => api.deepLink[command](scheme),
          command,
          scheme
        )
        expect(error).toMatch(/unsupported platform/i)
      }
    }
  )
})
