![biometric](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/biometric/banner.png)

Prompt the user for biometric authentication on Android and iOS.

| Platform | Supported |
| -------- | --------- |
| Linux    | x         |
| Windows  | x         |
| macOS    | x         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
# the plugin only supports Android and iOS
[target.'cfg(any(target_os = "android", target_os = "ios"))'.dependencies]
tauri-plugin-biometric = "2"
# alternatively with Git:
tauri-plugin-biometric = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

<!-- Add the branch for installations using git! -->

```sh
pnpm add @tauri-apps/plugin-biometric
# or
npm add @tauri-apps/plugin-biometric
# or
yarn add @tauri-apps/plugin-biometric
```

## Usage

First you need to register the core plugin with Tauri. The crate is empty on desktop targets, so the registration must be gated to mobile:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_biometric::init())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

On iOS, add the `NSFaceIDUsageDescription` key to your app's `Info.plist` (e.g. `src-tauri/Info.ios.plist`). Without it, `checkStatus()` reports Face ID devices as unavailable with the error "NSFaceIDUsageDescription is not in the app Info.plist":

```xml
<key>NSFaceIDUsageDescription</key>
<string>Authenticate to access your data</string>
```

Then allow the plugin's commands in your capability file, for example with the default permission set, which grants `authenticate` and `status`:

`src-tauri/capabilities/mobile.json`

```json
{
  "$schema": "../gen/schemas/mobile-schema.json",
  "identifier": "mobile-capability",
  "windows": ["main"],
  "platforms": ["iOS", "android"],
  "permissions": ["biometric:default"]
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { authenticate, checkStatus } from '@tauri-apps/plugin-biometric'

const status = await checkStatus()
if (status.isAvailable) {
  try {
    await authenticate('Open your wallet', { allowDeviceCredential: true })
  } catch (e) {
    // the user canceled, failed to authenticate, or biometry is unavailable
  }
} else {
  console.log(status.error, status.errorCode)
}
```

The same APIs are available in Rust through the `BiometricExt` trait:

```rust
#[cfg(mobile)]
fn unlock(app: &tauri::AppHandle) -> tauri_plugin_biometric::Result<()> {
    use tauri_plugin_biometric::{AuthOptions, BiometricExt};

    app.biometric().authenticate(
        "Open your wallet".into(),
        AuthOptions {
            allow_device_credential: true,
            ..Default::default()
        },
    )
}
```

### Security considerations

`authenticate` only tells your app that the user passed the system prompt. The result is not bound to any cryptographic key (no Android `CryptoObject` or iOS Keychain access control), Android accepts Class 2 ("weak") biometrics, and the result can be forged on rooted or jailbroken devices. Use it as a user-presence check, not as the only protection for secrets.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Contributed By

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="contributors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
      <td align="center" valign="middle">
        <a href="https://impierce.com" target="_blank">
            <img src="contributors/impierce.svg" alt="Impierce" width="283" height="90">
        </a>
      </td>
    </tr>
  </tbody>
</table>

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
