![secure-storage](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/secure-storage/banner.png)

Store secrets such as tokens and passwords in the operating system's secure storage.

| Platform | Supported | Backend                                                                                                            |
| -------- | --------- | ------------------------------------------------------------------------------------------------------------------ |
| Linux    | ✓         | [Secret Service](https://specifications.freedesktop.org/secret-service/latest/) (GNOME Keyring, KWallet, ...)      |
| Windows  | ✓         | Credential Manager                                                                                                 |
| macOS    | ✓         | Keychain                                                                                                           |
| Android  | ✓         | AES-256-GCM with a key held in the [Android Keystore](https://developer.android.com/privacy-and-security/keystore) |
| iOS      | ✓         | Keychain                                                                                                           |

Items are scoped to the application: on desktop and iOS they are stored under the app's identifier, on Android they live in the app's private storage.

## Platform notes

- **Linux:** a Secret Service provider must be running and unlocked. Calls fail on systems without one (minimal window managers, headless sessions, containers); the app still starts.
- **Windows:** the Credential Manager rejects values larger than 2560 bytes.
- **macOS:** the item is tied to the code signature of the app that created it, so unsigned or re-signed builds (including development builds) prompt for the login keychain password.
- **Android:** only the values are encrypted, the keys (item names) are stored in plain text. The encryption key never leaves the device, so values restored from a backup or transferred to another device can't be decrypted and are reported as missing.
- **iOS:** items are stored with `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`: they are not included in iCloud Keychain or restored on another device, and they survive uninstalling the app.

This plugin protects secrets at rest. Any webview granted its permissions can read every item the app stored, so only grant them to windows that need it.

## Install

_This plugin requires a Rust version of at least **1.90**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-secure-storage = "2.0.0"
# alternatively with Git:
tauri-plugin-secure-storage = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-secure-storage
# or
npm add @tauri-apps/plugin-secure-storage
# or
yarn add @tauri-apps/plugin-secure-storage
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_secure_storage::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Second, add the required permissions in the project:

`src-tauri/capabilities/default.json`

```json
  "permissions": [
    "secure-storage:default"
  ]
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { getItem, setItem, removeItem } from '@tauri-apps/plugin-secure-storage'

await setItem('access-token', 'eyJhbGciOi...')
const token = await getItem('access-token') // null when there is none
await removeItem('access-token')
```

The same APIs are available in Rust through the `SecureStorageExt` trait:

```rust
use tauri_plugin_secure_storage::SecureStorageExt;

tauri::Builder::default()
    .plugin(tauri_plugin_secure_storage::init())
    .setup(|app| {
        app.secure_storage().set("access-token", "eyJhbGciOi...")?;
        let token = app.secure_storage().get("access-token")?; // None when there is none
        app.secure_storage().remove("access-token")?;
        Ok(())
    })
```

The Rust methods block on the credential store (a D-Bus round trip on Linux, possibly a system prompt on macOS), so call them from a background thread rather than the main thread once the app is running.

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
