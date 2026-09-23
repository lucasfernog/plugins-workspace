// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

// Scanning and writing need a physical tag, so only the record builders (pure
// JS) and the "NFC unavailable" paths of the emulator/simulator are covered.

const utf8 = (text: string) => Array.from(new TextEncoder().encode(text))

describePlugin('nfc', { mobileOnly: true }, () => {
  it('builds records from strings and byte arrays', async () => {
    const records = await tauri((api) => [
      api.nfc.record(api.nfc.NFCTypeNameFormat.Media, 'text/plain', 'id', 'é'),
      api.nfc.record(api.nfc.NFCTypeNameFormat.Unknown, [], [1, 2], [255])
    ])
    expect(records).toEqual([
      {
        format: 2,
        kind: utf8('text/plain'),
        id: utf8('id'),
        payload: [0xc3, 0xa9]
      },
      { format: 5, kind: [], id: [1, 2], payload: [255] }
    ])
  })

  it('builds text records with the language code length in the status byte', async () => {
    const records = await tauri((api) => [
      api.nfc.textRecord('hello'),
      api.nfc.textRecord('olá', 'record-id', 'pt-BR')
    ])
    expect(records).toEqual([
      {
        format: 1,
        kind: [0x54],
        id: [],
        payload: [2, ...utf8('en'), ...utf8('hello')]
      },
      {
        format: 1,
        kind: [0x54],
        id: utf8('record-id'),
        payload: [5, ...utf8('pt-BR'), ...utf8('olá')]
      }
    ])
  })

  it('builds URI records with the abbreviated prefix code', async () => {
    const payloads = await tauri((api) =>
      [
        'https://www.tauri.app',
        'https://tauri.app/docs',
        'tel:+123',
        'urn:nfc:ext',
        'custom:thing'
      ].map((uri) => api.nfc.uriRecord(uri).payload)
    )
    expect(payloads).toEqual([
      [0x02, ...utf8('tauri.app')],
      [0x04, ...utf8('tauri.app/docs')],
      [0x05, ...utf8('+123')],
      [0x23, ...utf8('ext')],
      [0x00, ...utf8('custom:thing')]
    ])
  })

  it('reports whether NFC is available', async () => {
    expect(typeof (await tauri((api) => api.nfc.isAvailable()))).toBe('boolean')
  })

  it('rejects scan and write when NFC is unavailable', async function () {
    if (await tauri((api) => api.nfc.isAvailable())) {
      // a real NFC reader would wait for a tag here
      this.skip()
    }
    const scanError = await tauriError((api) => api.nfc.scan({ type: 'ndef' }))
    expect(scanError).toMatch(/NFC (reading )?unavailable/)
    const writeError = await tauriError((api) =>
      api.nfc.write([api.nfc.textRecord('hello')], { kind: { type: 'ndef' } })
    )
    expect(writeError).toMatch(/NFC (reading )?unavailable/)
  })
})
