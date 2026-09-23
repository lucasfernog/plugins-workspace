// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

// The driver launches the app without arguments, so the matches reflect the
// CLI definition in the example's `tauri.conf.json` with nothing set. The
// example only registers the plugin on desktop, so the suite is skipped on
// mobile (which has no command line to begin with).

describePlugin('cli', { desktopOnly: true }, () => {
  it('getMatches reports every defined argument as unset', async () => {
    const matches = await tauri((api) => api.cli.getMatches())
    expect(Object.keys(matches.args).sort()).toEqual([
      'config',
      'theme',
      'verbose'
    ])
    // flags resolve to `false`, arguments taking a value to `null`
    expect(matches.args.verbose).toEqual({ value: false, occurrences: 0 })
    expect(matches.args.config).toEqual({ value: null, occurrences: 0 })
    expect(matches.args.theme).toEqual({ value: null, occurrences: 0 })
  })

  it('getMatches reports no subcommand', async () => {
    const subcommand = await tauri(
      async (api) => (await api.cli.getMatches()).subcommand
    )
    expect(subcommand).toBeNull()
  })

  it('getMatchesFrom parses the given arguments', async () => {
    const matches = await tauri(
      (api, args) => api.cli.getMatchesFrom(args),
      ['api', '-c', 'app.conf', '--theme', 'dark', '-vv']
    )
    expect(matches.args.config).toEqual({ value: 'app.conf', occurrences: 1 })
    expect(matches.args.theme).toEqual({ value: 'dark', occurrences: 1 })
    // a flag can be repeated and counts its occurrences
    expect(matches.args.verbose).toEqual({ value: true, occurrences: 2 })
    expect(matches.subcommand).toBeNull()
  })

  it('getMatchesFrom resolves subcommands', async () => {
    const subcommand = await tauri(
      async (api, args) => (await api.cli.getMatchesFrom(args)).subcommand,
      ['api', 'update', '-b']
    )
    expect(subcommand?.name).toBe('update')
    expect(subcommand?.matches.args.background).toEqual({
      value: true,
      occurrences: 1
    })
  })

  it('getMatchesFrom rejects arguments the definition does not accept', async () => {
    expect(
      await tauriError(
        (api, args) => api.cli.getMatchesFrom(args),
        ['api', '--theme', 'blue']
      )
    ).toMatch(/invalid value/)
    expect(
      await tauriError(
        (api, args) => api.cli.getMatchesFrom(args),
        ['api', '--unknown']
      )
    ).toMatch(/unexpected argument/)
  })
})
