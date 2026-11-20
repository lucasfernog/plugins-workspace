// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

// Items go to the real credential store of the machine, under the example's
// identifier. On Linux that needs an unlocked Secret Service provider in the
// session (CI starts gnome-keyring for it).
const key = 'e2e-secure-storage'
const otherKey = 'e2e-secure-storage-other'

describePlugin('secure-storage', () => {
  const cleanup = async () => {
    await tauri(
      async (api, key, otherKey) => {
        await api.secureStorage.removeItem(key)
        await api.secureStorage.removeItem(otherKey)
      },
      key,
      otherKey
    )
  }

  before(cleanup)
  after(cleanup)

  it('getItem resolves to null for a missing item', async () => {
    expect(await tauri((api, key) => api.secureStorage.getItem(key), key)).toBe(
      null
    )
  })

  it('setItem stores, overwrites and keeps items apart', async () => {
    const result = await tauri(
      async (api, key, otherKey) => {
        await api.secureStorage.setItem(key, 's3cr3t')
        const first = await api.secureStorage.getItem(key)
        await api.secureStorage.setItem(key, 'ünïcödé 🔑')
        await api.secureStorage.setItem(otherKey, 'other')
        return {
          first,
          overwritten: await api.secureStorage.getItem(key),
          other: await api.secureStorage.getItem(otherKey)
        }
      },
      key,
      otherKey
    )
    expect(result).toEqual({
      first: 's3cr3t',
      overwritten: 'ünïcödé 🔑',
      other: 'other'
    })
  })

  it('removeItem deletes the item and ignores missing ones', async () => {
    const result = await tauri(async (api, key) => {
      await api.secureStorage.setItem(key, 'to be removed')
      await api.secureStorage.removeItem(key)
      const afterRemove = await api.secureStorage.getItem(key)
      // removing a missing item is not an error
      await api.secureStorage.removeItem(key)
      return afterRemove
    }, key)
    expect(result).toBe(null)
  })

  it('rejects an empty key', async () => {
    const error = await tauriError((api) => api.secureStorage.getItem(''))
    expect(error).toContain('must not be empty')
  })
})
