![plugin-process](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/process/banner.png)

This plugin provides APIs to access the current process. To spawn child processes, see the [`shell`](https://github.com/tauri-apps/tauri-plugin-shell) plugin.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | partial   |
| iOS      | x         |

The plugin can be registered on every platform, but on mobile:

- **Android**: `exit` works. `relaunch` terminates the app but cannot start it again, since Android apps are launched by the system, not by spawning their binary.
- **iOS**: iOS does not allow an app to quit programmatically, so both `exit` and `relaunch` only emit `RunEvent::ExitRequested` and the app keeps running.

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
tauri-plugin-process = "2.0.0"
# alternatively with Git:
tauri-plugin-process = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-process
# or
npm add @tauri-apps/plugin-process
# or
yarn add @tauri-apps/plugin-process
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { exit, relaunch } from '@tauri-apps/plugin-process'

// exit the app with the given status code (defaults to 0)
await exit(0)

// or restart the app
await relaunch()
```

Both go through `RunEvent::ExitRequested`, so the app can still prevent them with `api.prevent_exit()`.

From Rust you don't need this plugin: call `AppHandle::exit` and `AppHandle::request_restart` directly. The plugin only exposes them to JavaScript.

### Permissions

The `process:default` permission set allows both commands. Note that `relaunch()` calls the `restart` command, so its permission is `process:allow-restart`:

| JavaScript   | Permission              |
| ------------ | ----------------------- |
| `exit()`     | `process:allow-exit`    |
| `relaunch()` | `process:allow-restart` |

Any window or remote origin you grant these permissions can terminate or restart your app.

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
