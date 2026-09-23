// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'

// The feedback itself cannot be observed (and emulators and simulators have no
// haptic hardware), so this only checks that every command goes through the
// native implementation and settles. The functions never reject: they resolve
// to `{ status: 'ok', data }` or `{ status: 'error', error }`.
// The example grants the haptics permissions in its mobile capability.

describePlugin('haptics', { mobileOnly: true }, () => {
  it('vibrate resolves to an ok result', async () => {
    expect(await tauri((api) => api.haptics.vibrate(50))).toEqual({
      status: 'ok',
      data: null
    })
  })

  it('vibrate(0) stops the current vibration', async () => {
    const results = await tauri(async (api) => [
      await api.haptics.vibrate(5000),
      await api.haptics.vibrate(0)
    ])
    expect(results).toEqual([
      { status: 'ok', data: null },
      { status: 'ok', data: null }
    ])
  })

  it('impactFeedback accepts every style', async () => {
    const results = await tauri(async (api) => {
      const styles = ['light', 'medium', 'heavy', 'soft', 'rigid'] as const
      const out = []
      for (const style of styles) {
        out.push(await api.haptics.impactFeedback(style))
      }
      return out
    })
    expect(results).toEqual(Array(5).fill({ status: 'ok', data: null }))
  })

  it('notificationFeedback accepts every type', async () => {
    const results = await tauri(async (api) => {
      const types = ['success', 'warning', 'error'] as const
      const out = []
      for (const type of types) {
        out.push(await api.haptics.notificationFeedback(type))
      }
      return out
    })
    expect(results).toEqual(Array(3).fill({ status: 'ok', data: null }))
  })

  it('selectionFeedback resolves to an ok result', async () => {
    expect(await tauri((api) => api.haptics.selectionFeedback())).toEqual({
      status: 'ok',
      data: null
    })
  })

  it('resolves to an error result instead of rejecting on invalid input', async () => {
    const result = await tauri((api) =>
      // @ts-expect-error an unknown style
      api.haptics.impactFeedback('bogus')
    )
    expect(result.status).toBe('error')
    expect(result.status === 'error' && result.error).toMatch(
      /unknown variant `bogus`/
    )
  })
})
