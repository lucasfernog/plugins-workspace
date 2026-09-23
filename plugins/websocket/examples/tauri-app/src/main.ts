// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import WebSocket from '@tauri-apps/plugin-websocket'
import './style.css'

let ws: WebSocket | undefined

document.addEventListener('DOMContentLoaded', async () => {
  document.querySelector('#send')?.addEventListener('click', send)
  document.querySelector('#disconnect')?.addEventListener('click', disconnect)
  await connect()
})

function _updateResponse(returnValue: unknown) {
  const msg = document.createElement('p')
  msg.textContent =
    typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)
  document.querySelector('#response-container')?.appendChild(msg)
}

async function connect() {
  try {
    ws = await WebSocket.connect('ws://127.0.0.1:8080')
    _updateResponse('Connected')
    ws.addListener(_updateResponse)
  } catch (e) {
    _updateResponse(e)
  }
}

function send() {
  if (!ws) {
    _updateResponse('Not connected')
    return
  }
  ws.send(document.querySelector<HTMLInputElement>('#msg-input')?.value ?? '')
    .then(() => {
      _updateResponse('Message sent')
    })
    .catch(_updateResponse)
}

function disconnect() {
  if (!ws) {
    _updateResponse('Not connected')
    return
  }
  ws.disconnect()
    .then(() => {
      _updateResponse('Disconnected')
    })
    .catch(_updateResponse)
}

document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
  <div>
    <input id="msg-input" type="text" />
    <button id="send">send</button>
    <button id="disconnect">disconnect</button>
    <div id="response-container"></div>
  </div>
`
