![plugin-dialog](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/dialog/banner.png)

Native system dialogs for opening and saving files along with message dialogs.

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

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-dialog = "2.0.0"
# alternatively with Git:
tauri-plugin-dialog = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

### Linux XDG Desktop Portal Support

By default, this plugin uses gtk to show dialogs, however since `v2.5.0` you can switch to using [XDG Desktop Portal](https://flatpak.github.io/xdg-desktop-portal/) by adding the following to your `Cargo.toml` file:

```toml
[dependencies]
tauri-plugin-dialog = { version = "2.5.0", default-features = false, features = ["xdg-portal"] }
# alternatively with Git:
tauri-plugin-dialog = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2", default-features = false, features = ["xdg-portal"] }

```

Do note if you use the `xdg-portal` feature, you need to ensure that [`zenity`](https://gitlab.gnome.org/GNOME/zenity) and an [XDG Desktop Portal backend](https://flatpak.github.io/xdg-desktop-portal#using-portals) is installed with your program.

For more information, see [XDG Desktop Portal documentation](https://flatpak.github.io/xdg-desktop-portal/) and [`rfd` documentation](https://docs.rs/rfd/latest/rfd#xdg-desktop-portal-backend).

### JavaScript

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-dialog
# or
npm add @tauri-apps/plugin-dialog
# or
yarn add @tauri-apps/plugin-dialog
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { open, save, message, ask, confirm } from '@tauri-apps/plugin-dialog'

// pick a single file; resolves to `null` if the user cancels
const file = await open({
  multiple: false,
  filters: [{ name: 'Images', extensions: ['png', 'jpeg'] }]
})

// pick a path to save to
const path = await save({ defaultPath: 'notes.txt' })

// message dialogs
await message('File not found', { title: 'Tauri', kind: 'error' })
const yes = await ask('Discard the changes?', { kind: 'warning' })
const ok = await confirm('Are you sure?')
```

Or from Rust:

```rust
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .setup(|app| {
        app.dialog()
            .message("Continue?")
            .buttons(MessageDialogButtons::OkCancel)
            .show(|confirmed| println!("confirmed: {confirmed}"));
        app.dialog().file().pick_file(|path| {
            // `None` if the user cancelled
            println!("{path:?}");
        });
        Ok(())
    });
```

The `blocking_*` variants (for example `blocking_pick_file`) must not be called on the main thread,
where they would freeze the app.

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
