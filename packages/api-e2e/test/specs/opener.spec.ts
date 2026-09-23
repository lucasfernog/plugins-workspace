// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

/**
 * Clicks a link to the denied `https://denied.e2e.invalid/` with the given
 * modifier key, in the page. Resolves with whether the opener's injected click
 * handler cancelled the navigation and what it logged with `console.error`
 * once `open_url` rejected. The navigation is always cancelled afterwards, so
 * the page never leaves the app.
 */
function clickDeniedLink(modifier: 'ctrlKey' | 'metaKey') {
  return tauri(
    (_api, modifier) =>
      new Promise<{ prevented: boolean; errors: string[] }>((resolve) => {
        const link = document.createElement('a')
        link.href = 'https://denied.e2e.invalid/link'
        link.textContent = 'denied link'
        document.body.appendChild(link)

        const errors: string[] = []
        const consoleError = console.error
        console.error = (...args: unknown[]) => {
          errors.push(args.map(String).join(' '))
          consoleError.apply(console, args)
        }

        let prevented = false
        // registered after the plugin's listener on `window`, so it runs last
        const guard = (event: MouseEvent) => {
          prevented = event.defaultPrevented
          event.preventDefault()
        }
        window.addEventListener('click', guard)

        link.dispatchEvent(
          new MouseEvent('click', {
            bubbles: true,
            cancelable: true,
            button: 0,
            [modifier]: true
          })
        )

        const started = Date.now()
        const poll = () => {
          if (errors.length > 0 || Date.now() - started > 5000) {
            window.removeEventListener('click', guard)
            console.error = consoleError
            link.remove()
            resolve({ prevented, errors })
          } else {
            setTimeout(poll, 50)
          }
        }
        poll()
      }),
    modifier
  )
}

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

  // no `target="_blank"`: the example also registers the shell plugin, whose
  // own handler opens `_blank` links through `shell|open`
  it('logs why a Ctrl-clicked link could not be opened', async () => {
    const { prevented, errors } = await clickDeniedLink('ctrlKey')
    expect(prevented).toBe(true)
    expect(errors.join('\n')).toMatch(
      /Failed to open https:\/\/denied\.e2e\.invalid\/link.*Not allowed to open url/
    )
  })

  it('opens Cmd/Meta-clicked links like Ctrl-clicked ones', async () => {
    const { prevented, errors } = await clickDeniedLink('metaKey')
    expect(prevented).toBe(true)
    expect(errors.join('\n')).toMatch(/Not allowed to open url/)
  })
})
