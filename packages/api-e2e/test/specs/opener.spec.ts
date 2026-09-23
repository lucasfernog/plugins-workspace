// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauriError, describePlugin } from '../helpers/index.js'

// A successful open launches an external application (browser, file manager)
// the suite cannot control or close, so only the scope enforcement is covered.
// The example allows `mailto:`, `tel:`, `http(s)://` URLs (opener:default),
// `https://` URLs specifically with `inAppBrowser` (but denies
// `https://denied.e2e.invalid/*`), and paths under `$APPDATA`.

describePlugin('opener', () => {
  it('openUrl rejects URL schemes outside the scope', async () => {
    const message = await tauriError((api) =>
      api.opener.openUrl('ftp://example.com/file')
    )
    expect(message).toMatch(
      /Not allowed to open url ftp:\/\/example\.com\/file/
    )
  })

  it('openUrl rejects an app that is not in the scope for the URL', async () => {
    // `inAppBrowser` is only allowed for `https://` URLs
    const message = await tauriError((api) =>
      api.opener.openUrl('http://example.com', 'inAppBrowser')
    )
    expect(message).toMatch(/Not allowed to open url http:\/\/example\.com/)
  })

  it('openUrl deny rules apply to every app', async () => {
    // `https://denied.e2e.invalid/*` is denied without an `app`, while
    // `https://*` is allowed with `inAppBrowser`
    const message = await tauriError((api) =>
      api.opener.openUrl('https://denied.e2e.invalid/x', 'inAppBrowser')
    )
    expect(message).toMatch(
      /Not allowed to open url https:\/\/denied\.e2e\.invalid\/x/
    )
  })

  it('openUrl deny rules match normalized URLs', async () => {
    for (const url of [
      'https://DENIED.e2e.invalid/x',
      'https://denied.e2e.invalid:443/x',
      'https://denied.e2e.invalid./x'
    ]) {
      const message = await tauriError((api, u) => api.opener.openUrl(u), url)
      expect(message).toMatch(/Not allowed to open url/)
    }
  })

  it('openPath rejects paths outside the scope', async () => {
    const message = await tauriError(async (api) =>
      api.opener.openPath(await api.path.join(await api.path.homeDir(), 'e2e'))
    )
    expect(message).toMatch(/Not allowed to open path/)
  })

  it('revealItemInDir rejects paths that do not exist', async () => {
    const message = await tauriError(async (api) =>
      api.opener.revealItemInDir(
        await api.path.join(await api.path.appDataDir(), 'does-not-exist-e2e')
      )
    )
    expect(message).toMatch(/os error 2/)
  })
})
