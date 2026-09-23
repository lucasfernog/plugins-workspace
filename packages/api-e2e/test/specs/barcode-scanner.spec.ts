// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin, itOn } from '../helpers/index.js'

// Nothing can be put in front of the camera, so a scan never succeeds here.
// What is covered is that every call settles: the Android session is started
// with `autoGrantPermissions`, so `scan` opens the (emulated) camera there and
// must be rejected by `cancel`; the iOS simulator has no camera, so `scan`
// rejects straight away.

describePlugin('barcode-scanner', { mobileOnly: true }, () => {
  it('checkPermissions reports a permission state', async () => {
    const state = await tauri((api) => api.barcodeScanner.checkPermissions())
    expect(['granted', 'denied', 'prompt', 'prompt-with-rationale']).toContain(
      state
    )
  })

  it('cancel resolves when no scan is running', async () => {
    expect(await tauri((api) => api.barcodeScanner.cancel())).toBeNull()
  })

  itOn('android', 'cancel rejects the pending scan', async () => {
    const outcome = await tauri(async (api) => {
      const scan = api.barcodeScanner.scan().then(
        () => 'resolved',
        (error: unknown) => `rejected: ${String(error)}`
      )
      // let the camera start before cancelling
      await new Promise((resolve) => setTimeout(resolve, 1000))
      await api.barcodeScanner.cancel()
      return await Promise.race([
        scan,
        new Promise<string>((resolve) =>
          setTimeout(() => resolve('still pending after cancel'), 10_000)
        )
      ])
    })
    // `No camera available` on a device without any camera
    expect(outcome).toMatch(/^rejected: .*(cancelled|camera)/i)
  })

  itOn(
    'ios',
    'scan rejects on the simulator, which has no camera',
    async () => {
      const message = await tauriError((api) => api.barcodeScanner.scan())
      expect(message).toMatch(/No camera available/)
    }
  )
})
