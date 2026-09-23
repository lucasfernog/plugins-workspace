![plugin-websocket](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/websocket/banner.png)

Open a WebSocket connection using a Rust client in JS.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-websocket = "2"
# alternatively with Git:
tauri-plugin-websocket = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-websocket
# or
npm add @tauri-apps/plugin-websocket
# or
yarn add @tauri-apps/plugin-websocket
```

### TLS

`wss://` URLs need one of the TLS Cargo features:

- `rustls-tls` _(enabled by default)_: `rustls` with the WebPKI root certificates.
- `rustls-tls-native-roots`: `rustls` with the platform's native root certificates. Use it when the server's certificate is issued by a CA installed on the machine (e.g. a corporate CA).
- `native-tls`: the platform's TLS library.
- `native-tls-vendored`: `native-tls` with a vendored OpenSSL.

Plain `ws://` works without any of them. For custom certificates (e.g. a self-signed server), pass your own [`Connector`](https://docs.rs/tokio-tungstenite/latest/tokio_tungstenite/enum.Connector.html) with `tauri_plugin_websocket::Builder::new().tls_connector(connector).build()` instead of `init()`.

## Usage

First you need to register the plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_websocket::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Then allow the plugin's commands in one of your [capabilities](https://v2.tauri.app/security/capabilities/), e.g. `src-tauri/capabilities/default.json`:

```json
{
  "permissions": ["websocket:default"]
}
```

`websocket:default` allows opening connections (`allow-connect`) and sending messages on them (`allow-send`).

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import WebSocket from '@tauri-apps/plugin-websocket'

const ws = await WebSocket.connect('wss://example.com', {
  // optional: extra headers for the connect (upgrade) request
  headers: { Authorization: 'Bearer <token>' }
})

const removeListener = ws.addListener((message) => {
  if (typeof message === 'string') {
    // the stream failed; errors are delivered as plain strings
    console.error(message)
    return
  }
  switch (message.type) {
    case 'Text':
      console.log(message.data)
      break
    case 'Binary':
      console.log(message.data) // a number[] of bytes
      break
    case 'Close':
      console.log('closed', message.data?.code, message.data?.reason)
      break
  }
})

await ws.send('Hello World') // a Text message
await ws.send([1, 2, 3]) // a Binary message
await ws.send({ type: 'Ping', data: [] }) // any Message, e.g. Ping, Pong or Close

removeListener()
await ws.disconnect() // sends a Close frame with code 1000
```

### Things to know

- The connection is made from Rust, so the webview's Content Security Policy (`connect-src`) does not apply and there is no URL scope: any capability granting `websocket:default` can connect to any `ws://` or `wss://` URL, including local network services, with any handshake headers (including `Origin`).
- There are no open/error/close events besides the messages passed to `addListener`, and connections are not closed when the page reloads; call `disconnect()` when you are done with a connection.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
