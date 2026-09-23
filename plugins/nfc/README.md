![NFC](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/nfc/banner.png)

Read and write NFC tags on Android and iOS.

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
[dependencies]
tauri-plugin-nfc = "2.0.0"
# alternatively with Git:
tauri-plugin-nfc = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

<!-- Add the branch for installations using git! -->

```sh
pnpm add @tauri-apps/plugin-nfc
# or
npm add @tauri-apps/plugin-nfc
# or
yarn add @tauri-apps/plugin-nfc
```

## Platform setup

### iOS

- Add the `NFCReaderUsageDescription` key to your `src-tauri/Info.ios.plist` file, describing why the app reads or writes NFC tags. Without it `isAvailable()` resolves to `false` and `scan()`/`write()` reject.

  ```xml
  <key>NFCReaderUsageDescription</key>
  <string>Read and write NFC tags</string>
  ```

- Add the "Near Field Communication Tag Reading" capability to the app target in Xcode ("Signing & Capabilities" tab), or add the entitlement to `src-tauri/gen/apple/<app-name>_iOS/<app-name>_iOS.entitlements`. The `TAG` format is required by the `tag` scan kind:

  ```xml
  <key>com.apple.developer.nfc.readersession.formats</key>
  <array>
    <string>TAG</string>
  </array>
  ```

- The `tag` scan kind polls ISO 14443 (MIFARE, ISO 7816) and ISO 15693 tags. To detect ISO 7816 tags, also list the application identifiers your app selects under the `com.apple.developer.nfc.readersession.iso7816.select-identifiers` key of the `Info.ios.plist` file.

### Android

The plugin adds the `android.permission.NFC` permission to the app manifest, and adds `NDEF_DISCOVERED`, `TECH_DISCOVERED` and `TAG_DISCOVERED` intent filters to the main activity. This means that tapping an NFC tag can open the app even when it is not scanning.

If the app cannot work without NFC, also declare the feature in `src-tauri/gen/android/app/src/main/AndroidManifest.xml` so that app stores hide it on devices without NFC:

```xml
<uses-feature android:name="android.hardware.nfc" android:required="true" />
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // the plugin only exists on Android and iOS
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_nfc::init())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

The crate is empty on desktop platforms, so the registration must be gated behind `#[cfg(mobile)]` for the app to keep compiling there.

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { scan, textRecord, write } from '@tauri-apps/plugin-nfc'

// read the NDEF records of a tag
const tag = await scan({ type: 'ndef' })

// scan a tag and write a message to it
await write([textRecord('Tauri is awesome!')], { kind: { type: 'ndef' } })

// or keep the session of a scan alive to write to the tag that was just read
await scan({ type: 'tag' }, { keepSessionAlive: true })
await write([textRecord('Tauri is awesome!')])
```

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
