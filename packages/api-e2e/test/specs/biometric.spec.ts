// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, itOn } from '../helpers/index.js'

// A successful `authenticate` needs a biometric (or passcode) the driver
// cannot provide, so only `checkStatus` and the no-UI rejection path are
// covered. The example grants `biometric:allow-status` and
// `biometric:allow-authenticate` in its mobile capability.

describePlugin('biometric', { mobileOnly: true }, () => {
  it('exposes every BiometryType value', async () => {
    const values = await tauri((api) => ({
      None: api.biometric.BiometryType.None,
      TouchID: api.biometric.BiometryType.TouchID,
      FaceID: api.biometric.BiometryType.FaceID,
      Iris: api.biometric.BiometryType.Iris,
      OpticID: api.biometric.BiometryType.OpticID
    }))
    expect(values).toEqual({
      None: 0,
      TouchID: 1,
      FaceID: 2,
      Iris: 3,
      OpticID: 4
    })
  })

  it('checkStatus reports availability and the biometry type', async () => {
    const status = await tauri((api) => api.biometric.checkStatus())
    expect(typeof status.isAvailable).toBe('boolean')
    expect([0, 1, 2, 3, 4]).toContain(status.biometryType)
    if (!status.isAvailable) {
      expect(typeof status.error).toBe('string')
      expect([
        'biometryNotAvailable',
        'biometryNotEnrolled',
        'biometryLockout',
        'passcodeNotSet',
        'invalidContext',
        'notInteractive'
      ]).toContain(status.errorCode)
    }
  })

  it('checkStatus is stable across calls', async () => {
    // iOS used to cache the status at load; it is now evaluated on every call
    const [first, second] = await tauri(async (api) => [
      await api.biometric.checkStatus(),
      await api.biometric.checkStatus()
    ])
    expect(second).toEqual(first)
  })

  // On iOS, `authenticate` without device credentials rejects up front with
  // the status error when biometry is unavailable, without showing any UI.
  // A simulator with an enrolled Face ID would show the prompt, so the call is
  // only made when the status says biometry is unavailable.
  itOn(
    'ios',
    'authenticate rejects without a prompt when biometry is unavailable',
    async () => {
      const result = await tauri(async (api) => {
        const status = await api.biometric.checkStatus()
        if (status.isAvailable) {
          return { skipped: true }
        }
        try {
          await api.biometric.authenticate('e2e', {
            allowDeviceCredential: false
          })
          return { skipped: false, rejected: false }
        } catch {
          return { skipped: false, rejected: true }
        }
      })
      if (!result.skipped) {
        expect(result.rejected).toBe(true)
      }
    }
  )
})
