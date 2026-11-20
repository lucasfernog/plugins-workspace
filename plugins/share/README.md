# Share

Share text, links and files through the system share sheet.

| Platform | Supported |
| -------- | --------- |
| Linux    | x         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

Linux has no system share sheet, so sharing fails there with an error.

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
tauri-plugin-share = "2.0.0"
# alternatively with Git:
tauri-plugin-share = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

<!-- Add the branch for installations using git! -->

```sh
pnpm add @tauri-apps/plugin-share
# or
npm add @tauri-apps/plugin-share
# or
yarn add @tauri-apps/plugin-share
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_share::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Second, add the required permissions in the project:

`src-tauri/capabilities/default.json`

```json
  "permissions": ["share:default"]
```

`share:default` lets the webview share text and URLs. To share files from JavaScript, also list the files the webview may share as the scope of the `share:allow-share` permission:

```json
  "permissions": [
    {
      "identifier": "share:allow-share",
      "allow": [{ "path": "$APPDATA/**" }]
    }
  ]
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { share } from '@tauri-apps/plugin-share'
import { appDataDir, join } from '@tauri-apps/api/path'

// text and links
await share({
  title: 'Tauri',
  text: 'Build smaller, faster and more secure apps',
  url: 'https://tauri.app'
})

// files, anchoring the share sheet to the button that was clicked
button.addEventListener('click', async () => {
  const report = await join(await appDataDir(), 'report.pdf')
  await share({ files: [report] }, { anchor: button.getBoundingClientRect() })
})
```

`share` resolves once the share sheet is shown. Whether the user completed the share or dismissed the sheet is not reported, because most platforms do not tell.

### Usage from Rust

The Rust API does not check the scope:

```rust
use tauri::Manager;
use tauri_plugin_share::{ShareData, ShareExt, ShareOptions};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_share::init())
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            app.share().share(
                &window,
                ShareData::new().text("Hello!").url("https://tauri.app"),
                ShareOptions::new(),
            )?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Platform notes

- **Android:** files must be inside one of the app's directories (the data and cache directories and their external storage counterparts). They are handed to the receiving app through a `FileProvider` the plugin adds to the app manifest, with the `${applicationId}.share.fileprovider` authority. The text and the URL are sent as a single text, with the URL on its own line.
- **iOS and macOS:** the share sheet is a popover on iPad and macOS. Pass `anchor` to attach it to the element that triggered it. It defaults to the center of the window.
- **Windows:** the share dialog needs a title, which defaults to the app name.
- **`title`:** shown by the share sheet on Android and Windows, used as the subject by apps that have one (such as email clients) on Android and iOS, and ignored on macOS.

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
