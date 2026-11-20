// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauriError, describePlugin } from '../helpers/index.js'

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
})
