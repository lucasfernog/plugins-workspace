![Barcode Scanner](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/barcode-scanner/banner.png)

Allows your mobile application to use the camera to scan QR codes, EAN-13 and other kinds of barcodes.

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
tauri-plugin-barcode-scanner = "2.0.0"
# alternatively with Git:
tauri-plugin-barcode-scanner = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-barcode-scanner
# or
npm add @tauri-apps/plugin-barcode-scanner
# or
yarn add @tauri-apps/plugin-barcode-scanner
```

## Setting up

### iOS

Apple requires a privacy description to be specified in `Info.plist` for camera access. Without it, `scan` rejects:

- `NSCameraUsageDescription`

The iOS Simulator has no camera, so `scan` rejects there as well.

### Android

This plugin automatically adds the following to your `AndroidManifest.xml` file:

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.VIBRATE" />

<uses-feature android:name="android.hardware.camera.any" />
```

The `uses-feature` declaration is required by default, so the Google Play Store does not offer your app to devices without a camera. If scanning is optional in your app, replace it in your own `AndroidManifest.xml`:

```xml
<uses-feature
  android:name="android.hardware.camera.any"
  android:required="false"
  tools:node="replace" />
```

(`tools` is the `http://schemas.android.com/tools` namespace.)

## Usage

First you need to register the core plugin with Tauri. The plugin only exists on Android and iOS, so its registration must be gated with `#[cfg(mobile)]` for the app to build on desktop:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings.

`scan` rejects if the camera permission has not been granted, so check it and request it first:

```javascript
import {
  checkPermissions,
  requestPermissions,
  openAppSettings
} from '@tauri-apps/plugin-barcode-scanner'

let permission = await checkPermissions()
if (permission === 'prompt' || permission === 'prompt-with-rationale') {
  permission = await requestPermissions()
}
if (permission !== 'granted') {
  // the user denied the permission, only the system settings can grant it now
  await openAppSettings()
}
```

Then start scanning:

```javascript
import { scan, Format } from '@tauri-apps/plugin-barcode-scanner'

const { content, format } = await scan({
  windowed: true,
  formats: [Format.QRCode]
})
```

By default the camera preview is shown full screen on top of the webview, so your UI is hidden until the scan completes or is cancelled. With `windowed: true` the preview is placed _behind_ the webview instead, and the webview is made transparent, so you can draw your own UI (a viewfinder frame, a cancel button) over the camera. For the camera to be visible, the page itself must be transparent where the camera should show through, for example:

```css
html,
body {
  background: transparent;
}
```

The webview background is restored when the scan ends. On Android, if the webview had no background of its own, it is set to white.

## Permissions

By default no plugin command is allowed. Add the `barcode-scanner:default` permission set to a capability to allow all of them, which includes opening the app settings and the Android-only `vibrate` command:

`src-tauri/capabilities/mobile.json`

```json
{
  "$schema": "../gen/schemas/mobile-schema.json",
  "identifier": "mobile-capability",
  "windows": ["main"],
  "platforms": ["iOS", "android"],
  "permissions": ["barcode-scanner:default"]
}
```

To allow only some of them, use the individual permissions instead: `barcode-scanner:allow-scan`, `barcode-scanner:allow-cancel`, `barcode-scanner:allow-check-permissions`, `barcode-scanner:allow-request-permissions`, `barcode-scanner:allow-open-app-settings` and `barcode-scanner:allow-vibrate`. See [the permission reference](./permissions/autogenerated/reference.md) for details.

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
