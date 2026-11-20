// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauriError, describePlugin, itOn } from '../helpers/index.js'

// A successful share opens the system share sheet, which the suite cannot
// control or close, so only the calls rejected before that are covered.
// The example allows sharing files under `$APPDATA`.

describePlugin('share', () => {
  it('rejects when there is nothing to share', async () => {
    const message = await tauriError((api) =>
      api.share.share({ title: 'only a title' })
    )
    expect(message).toMatch(/nothing to share/)
  })

  it('rejects files outside the scope', async () => {
    const message = await tauriError(async (api) =>
      api.share.share({
        files: [await api.path.join(await api.path.homeDir(), 'e2e')]
      })
    )
    expect(message).toMatch(/Not allowed to share file/)
  })

  it('rejects when one of several files is outside the scope', async () => {
    const message = await tauriError(async (api) =>
      api.share.share({
        text: 'e2e',
        files: [
          await api.path.join(await api.path.appDataDir(), 'e2e.txt'),
          await api.path.join(await api.path.homeDir(), 'e2e')
        ]
      })
    )
    expect(message).toMatch(/Not allowed to share file/)
  })

  // Linux has no system share sheet, so there the calls that pass the checks
  // above can be covered too: they fail instead of opening native UI.
  itOn('linux', 'rejects text and URLs on Linux', async () => {
    const message = await tauriError((api) =>
      api.share.share({ text: 'e2e', url: 'https://tauri.app' })
    )
    expect(message).toMatch(/not supported on the current platform/)
  })

  itOn('linux', 'lets files inside the scope through', async () => {
    const message = await tauriError(async (api) =>
      api.share.share({
        files: [await api.path.join(await api.path.appDataDir(), 'e2e.txt')]
      })
    )
    // past the scope check, so it is the platform that refuses
    expect(message).toMatch(/not supported on the current platform/)
  })
})
