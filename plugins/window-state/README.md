![plugin-window-state](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/window-state/banner.png)

Save window positions and sizes and restore them when the app is reopened.

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
tauri-plugin-window-state = "2.0.0"
# alternatively with Git:
tauri-plugin-window-state = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-window-state
# or
npm add @tauri-apps/plugin-window-state
# or
yarn add @tauri-apps/plugin-window-state
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all windows will remember their state when the app is being closed and will restore to their previous state on the next launch.

Optionally you can also tell the plugin to save the state of all open windows to disk by using the `save_window_state()` method exposed by the `AppHandleExt` trait:

```rust
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

// `tauri::AppHandle` now has the following additional method
// (from the `setup` hook, use `app.handle().save_window_state(...)`)
app.save_window_state(StateFlags::all())?; // will save the state of all open windows to disk
```

or through JavaScript

```javascript
import { saveWindowState, StateFlags } from '@tauri-apps/plugin-window-state'

await saveWindowState(StateFlags.ALL)
```

To manually restore a window's state you can call the `restore_state()` method exposed by the `WindowExt` trait:

```rust
use tauri_plugin_window_state::{WindowExt, StateFlags};

// all `Window` types now have the following additional method
window.restore_state(StateFlags::all())?; // will restore the window's saved state
```

or through JavaScript

```javascript
import {
  restoreStateCurrent,
  StateFlags
} from '@tauri-apps/plugin-window-state'

await restoreStateCurrent(StateFlags.ALL)
```

### Configuration

The plugin `Builder` controls what is saved and for which windows:

```rust
use tauri_plugin_window_state::StateFlags;

tauri_plugin_window_state::Builder::new()
    // what to save and restore (default: `StateFlags::all()`)
    .with_state_flags(StateFlags::SIZE | StateFlags::POSITION)
    // file name, relative to the app config directory (default: `.window-state.json`)
    .with_filename("window-state.json")
    // windows the plugin ignores entirely, e.g. a splash screen
    .with_denylist(&["splashscreen"])
    // or decide per label: return `false` to ignore the window
    .with_filter(|label| !label.starts_with("popup-"))
    // track "editor" but don't restore it on creation; call `restore_state` yourself
    .skip_initial_state("editor")
    // share one saved state between windows, e.g. "editor-1" and "editor-2"
    .map_label(|label| if label.starts_with("editor-") { "editor" } else { label })
    .build()
```

`with_denylist` replaces the list set by a previous call. A label excluded by `with_denylist` or `with_filter` is not restored, tracked or saved.

To avoid a window flashing at its default size and position before its state is restored, create it hidden (`"visible": false` in `tauri.conf.json`, or `.visible(false)` on the window builder). With `StateFlags::VISIBLE` (included in the default flags) the plugin shows and focuses every window it restores, unless it was saved as hidden, including one that has no saved state yet. Windows you want to keep hidden should therefore be excluded, or use flags without `VISIBLE`.

### Permissions

The JavaScript API needs the plugin's permissions in one of your [capabilities](https://v2.tauri.app/security/capabilities/). `window-state:default` allows all of its commands (`save_window_state`, `restore_state` and `filename`):

`src-tauri/capabilities/default.json`

```json
{
  "permissions": ["window-state:default"]
}
```

`restore_state` accepts the label of any window in the app, not only the calling one, and restoring with `StateFlags.VISIBLE` shows and focuses that window. Grant the individual `window-state:allow-*` permissions instead of the default set if a webview should not be able to do that.

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
