// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Webview script injected by the shell plugin on every page load.
 *
 * It installs a `click` listener on `document.body` that intercepts clicks on
 * `<a target="_blank">` elements whose `href` starts with `http://`, `https://`,
 * `mailto:` or `tel:`, cancels the navigation and opens the link with the
 * system's default application through the `plugin:shell|open` command instead.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

// open <a href="..."> links with the API
function openLinks(): void {
  document.querySelector('body')?.addEventListener('click', function (e) {
    // another handler already took care of this click
    if (e.defaultPrevented) return
    let target: HTMLElement | null = e.target as HTMLElement
    while (target) {
      if (target.matches('a')) {
        const t = target
        if (
          t.href !== ''
          && ['http://', 'https://', 'mailto:', 'tel:'].some((v) =>
            t.href.startsWith(v)
          )
          && t.target === '_blank'
        ) {
          invoke('plugin:shell|open', {
            path: t.href
          }).catch((error: unknown) => {
            // e.g. `shell:allow-open` is not granted to this webview
            console.error(error)
          })
          e.preventDefault()
        }
        break
      }
      target = target.parentElement
    }
  })
}

if (
  document.readyState === 'complete'
  || document.readyState === 'interactive'
) {
  openLinks()
} else {
  window.addEventListener('DOMContentLoaded', openLinks, true)
}
