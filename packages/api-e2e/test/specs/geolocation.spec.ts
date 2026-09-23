// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, itOn } from '../helpers/index.js'

// The app starts without the location permission, and anything that asks for
// it (`requestPermissions`, and `getCurrentPosition`/`watchPosition` on iOS)
// puts up a system dialog the session would then block on, so only calls that
// never prompt are covered. On Android, `watchPosition` does not prompt: it
// either starts watching or fails right away without the permission.

const PERMISSION_STATES = [
  'granted',
  'denied',
  'prompt',
  'prompt-with-rationale'
]

describePlugin('geolocation', { mobileOnly: true }, () => {
  it('checkPermissions reports a state for both aliases', async () => {
    const result = await tauri(async (api) => {
      try {
        const status = await api.geolocation.checkPermissions()
        return {
          ok: true,
          location: status.location,
          coarseLocation: status.coarseLocation
        }
      } catch (error) {
        return { ok: false, error: String(error) }
      }
    })
    if (result.ok) {
      expect(PERMISSION_STATES).toContain(result.location)
      expect(PERMISSION_STATES).toContain(result.coarseLocation)
    } else {
      // Rejects when the device's location services are turned off.
      expect(result.error).toMatch(/location services/i)
    }
  })

  it('clearWatch accepts an id that is not being watched', async () => {
    const result = await tauri(async (api) => {
      await api.geolocation.clearWatch(987654)
      return 'cleared'
    })
    expect(result).toBe('cleared')
  })

  itOn(
    'android',
    'watchPosition with partial options settles and its id can be cleared',
    async () => {
      const result = await tauri(async (api) => {
        const outcome = await Promise.race([
          api.geolocation
            .watchPosition({ enableHighAccuracy: false }, () => {})
            .then(
              (id) => ({ state: 'resolved', id, error: '' }),
              (error: unknown) => ({
                state: 'rejected',
                id: -1,
                error: String(error)
              })
            ),
          new Promise<{ state: string; id: number; error: string }>((resolve) =>
            setTimeout(
              () => resolve({ state: 'pending', id: -1, error: '' }),
              15_000
            )
          )
        ])
        if (outcome.state === 'resolved') {
          await api.geolocation.clearWatch(outcome.id)
        }
        return outcome
      })
      // It used to never resolve on Android.
      expect(result.state).not.toBe('pending')
      if (result.state === 'resolved') {
        expect(typeof result.id).toBe('number')
      } else {
        // Without the permission it may fail, but the partial options must be
        // accepted rather than failing to deserialize.
        expect(result.error).not.toMatch(/missing field|invalid type/)
      }
    }
  )
})
