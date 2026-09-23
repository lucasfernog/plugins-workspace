![plugin-autostart](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/autostart/banner.png)

Automatically launch your application at startup.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | x         |
| iOS      | x         |

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
tauri-plugin-autostart = "2.0.0"
# alternatively with Git:
tauri-plugin-autostart = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-autostart
# or
npm add @tauri-apps/plugin-autostart
# or
yarn add @tauri-apps/plugin-autostart
```

## Usage

First you need to register the core plugin with Tauri. The plugin only supports desktop platforms, so gate it with `#[cfg(desktop)]` (and add the dependency under `[target.'cfg(any(target_os = "macos", windows, target_os = "linux"))'.dependencies]` if you also build for mobile):

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(
                tauri_plugin_autostart::Builder::new()
                    // arguments passed to your app when it is launched at login
                    .args(["--autostarted"])
                    // name of the autostart entry, defaults to the `productName`
                    .app_name("My Custom Name")
                    .build(),
            )?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Things to know:

- The registered program is the executable that is running when `enable()` is called, so enabling autostart from `tauri dev` registers the development binary. On Linux the AppImage is registered when the app runs as one.
- Pass a flag such as `--autostarted` with `args` and check it with `std::env::args()` (or the CLI plugin) to tell whether your app was launched at login, for example to start minimized.
- The entry is named after `app_name`. If you change it (or your `productName`) in a later version, the entry registered under the old name stays behind, so disable autostart before renaming. Two apps with the same name overwrite each other's entry; consider using your bundle identifier.

### macOS launchers

On macOS the plugin can register the app in two ways, chosen with `Builder::macos_launcher` (only available on macOS, so gate the call with `#[cfg(target_os = "macos")]`):

- `MacosLauncher::LaunchAgent` (the default) writes `~/Library/LaunchAgents/<app name>.plist`, which starts the app's executable with the configured arguments. macOS 13+ shows a "Background Items Added" notification when it is enabled.
- `MacosLauncher::AppleScript` adds a login item through the "System Events" application. The login item opens the `.app` bundle and is named after it, so `app_name` is ignored, and only the `--hidden` and `--minimized` arguments have an effect (either one hides the app). macOS asks the user for the Automation permission the first time; set `NSAppleEventsUsageDescription` in your `Info.plist` to explain why.

```rust
let mut autostart = tauri_plugin_autostart::Builder::new();
#[cfg(target_os = "macos")]
{
    autostart = autostart.macos_launcher(tauri_plugin_autostart::MacosLauncher::AppleScript);
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'

await enable()

console.log(`registered for autostart? ${await isEnabled()}`)

await disable()
```

## Permissions

By default the plugin's commands are blocked. The `autostart:default` permission allows `enable`, `disable` and `isEnabled`; add it (or the individual `autostart:allow-*` permissions) to your capability:

`src-tauri/capabilities/default.json`

```json
{
  "permissions": ["autostart:default"]
}
```

Only grant it to windows you trust: a page allowed to call `enable()` can make your app start at every login.

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
