![plugin-positioner](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/positioner/banner.png)

Position your windows at well-known locations.

This plugin is a port of [electron-positioner](https://github.com/jenslind/electron-positioner) for Tauri.

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
tauri-plugin-positioner = "2.0.0"
# alternatively with Git:
tauri-plugin-positioner = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

The positions relative to the tray icon (`TrayLeft`, `TrayCenter`, …) need the `tray-icon` Cargo feature:

```toml
[dependencies]
tauri-plugin-positioner = { version = "2.0.0", features = ["tray-icon"] }
```

Without it, only the screen positions are available: using a `Tray*` position fails, and `moveWindowConstrained` and `handleIconState` are not available.

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-positioner
# or
npm add @tauri-apps/plugin-positioner
# or
yarn add @tauri-apps/plugin-positioner
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri::tray::TrayIconBuilder;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        // This is required to get tray-relative positions to work
        .setup(|app| {
            // note that this will create a new TrayIcon
            TrayIconBuilder::new()
                .on_tray_icon_event(|app, event| {
                    tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Alternatively, you may handle the tray events through JavaScript. Register the plugin as previously noted.

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

And in JavaScript, the `action` passed to the TrayIcon should include the handler.

```typescript
import { TrayIcon, type TrayIconEvent } from "@tauri-apps/api/tray";
import {
  moveWindow,
  Position,
  handleIconState,
} from "@tauri-apps/plugin-positioner";

const action = async (event: TrayIconEvent) => {
  // add the handle in the action to update the state
  await handleIconState(event);

  if (event.type === "Click") {
    // note this option requires enabling the `tray-icon`
    //   feature in the Cargo.toml
    await moveWindow(Position.TrayLeft);
  }
};

const tray = await TrayIcon.new({ id: "main", action });
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { moveWindow, Position } from '@tauri-apps/plugin-positioner'

await moveWindow(Position.TopRight)
```

### Tray positions

The `Tray*` positions are computed from the tray icon's last reported position, which is recorded by `on_tray_event` (Rust) or `handleIconState` (JavaScript) on click, enter, leave and move events. Until one of those events was forwarded, moving a window to a tray position fails with `Tray position not set`.

- On Windows and macOS, `TrayLeft`, `TrayRight` and `TrayCenter` put the window above the icon, and move it below the icon when there is no room above (on Windows the window starts right below the icon, on macOS it starts at the icon's top edge).
- On Linux, Tauri does not emit tray icon events, so the tray positions can't be used.
- `moveWindowConstrained` (Rust: `move_window_constrained`) additionally keeps tray-positioned windows inside the tray icon's monitor.

### Permissions

The `positioner:default` permission set allows `moveWindow`, `moveWindowConstrained` and `handleIconState`. Add it to your capability, for example `src-tauri/capabilities/default.json`:

```json
{
  "permissions": ["positioner:default"]
}
```

### Rust only

If you only move windows from Rust code, you can use the `WindowExt` trait extension, which is implemented for `WebviewWindow` and `Window`:

```rust
use tauri::Manager;
use tauri_plugin_positioner::{Position, WindowExt};

let win = app.get_webview_window("main").unwrap();
win.move_window(Position::TopRight)?;
```

Registering the plugin is not needed for this **unless** the `tray-icon` feature is enabled: the plugin stores the tray icon's position, and with the feature enabled, moving a window without the plugin registered panics, even for screen positions.

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

Code: (c) 2021 - Jonas Kruckenberg. 2021 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
