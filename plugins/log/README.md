![plugin-log](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/log/banner.png)

Configurable logging for your Tauri app.

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
tauri-plugin-log = "2.0.0"
# alternatively with Git:
tauri-plugin-log = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

The crate has the following optional Cargo features:

- `colored`: enables `Builder::with_colors`, which colors the log level with ANSI escape codes.
- `tracing`: log records sent from JavaScript are also emitted as [`tracing`](https://docs.rs/tracing) events.

Then you can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-log
# or
npm add @tauri-apps/plugin-log
# or
yarn add @tauri-apps/plugin-log
```

## Usage

First, you should enable the `log:default` capability:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": ["core:default", "opener:default", "log:default"]
}
```

Then, you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri_plugin_log::{Target, TargetKind};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
            Target::new(TargetKind::Webview),
        ]).build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { trace, info, error, attachConsole } from '@tauri-apps/plugin-log'

// with TargetKind::Webview enabled this function will print logs to the browser console
const detach = await attachConsole()

trace('Trace')
info('Info')
error('Error')

// detach the browser console from the log stream
detach()
```

To log from rust code, add the log crate to your `Cargo.toml`:

```toml
[dependencies]
log = "^0.4"
```

Now, you can use the macros provided by the log crate to log messages from your backend. See the [docs](https://docs.rs/log/latest) for more details. The plugin also re-exports the crate as `tauri_plugin_log::log`.

## Configuration

### Targets

By default the plugin logs to the `Stdout` and `LogDir` targets. `Builder::target` adds a target to these defaults, while `Builder::targets` and `Builder::clear_targets` replace them.

| Target                       | Destination                                                                                                                                                                                                                      |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Stdout` / `Stderr`          | The terminal on desktop, logcat on Android and the unified logging system (`os_log`) on iOS. On iOS, records are only written in debug builds and long messages are truncated to 4068 characters.                                |
| `LogDir { file_name }`       | `{file_name}.log` in the platform's log directory: `$XDG_DATA_HOME/{identifier}/logs` on Linux, `~/Library/Logs/{identifier}` on macOS and iOS, `%LOCALAPPDATA%\{identifier}\logs` on Windows and `{configDir}/logs` on Android. |
| `Folder { path, file_name }` | `{file_name}.log` in the given directory, which is created if needed.                                                                                                                                                            |
| `Webview`                    | Emits every record as a `log://log` event. Subscribe to it with `attachLogger` or `attachConsole` from JavaScript.                                                                                                               |
| `Dispatch(fern::Dispatch)`   | A custom [`fern`](https://docs.rs/fern) output.                                                                                                                                                                                  |

`file_name` defaults to the app's product name. Anything after the last `.` in the name is replaced by `log`, so avoid dots in file names: `app.webview` and `app.rust` would both write to `app.log`.

Every target accepts its own filters and formatter. For example, to write frontend and backend logs to separate files:

```rust
use tauri_plugin_log::{Target, TargetKind, WEBVIEW_TARGET};

tauri_plugin_log::Builder::new()
    .targets([
        Target::new(TargetKind::Stdout),
        Target::new(TargetKind::LogDir { file_name: Some("webview".into()) })
            .filter(|metadata| metadata.target().starts_with(WEBVIEW_TARGET)),
        Target::new(TargetKind::LogDir { file_name: Some("rust".into()) })
            .filter(|metadata| !metadata.target().starts_with(WEBVIEW_TARGET)),
    ])
    .build()
```

Records logged from JavaScript use the `webview` target, followed by `::{function}@{file}:{line}:{column}` when the caller's location could be read from the stack trace.

### Log files

When a log file grows past `Builder::max_file_size` (40,000 bytes by default), it is rotated according to `Builder::rotation_strategy`:

- `RotationStrategy::KeepOne` (default): the file is deleted and a new one is started.
- `RotationStrategy::KeepAll`: the file is renamed to `{file_name}_{date}.log` and kept forever.
- `RotationStrategy::KeepSome(n)`: like `KeepAll`, but only the `n` most recent archives are kept. `n` must be at least 1.

`Builder::file_open_strategy(FileOpenStrategy::Rotate)` starts a new file every time the app starts, instead of appending to the previous one (`FileOpenStrategy::Append`, the default).

### Format

On desktop, the default format is `[date][time][target][level] message`, with the time in UTC. On Android and iOS, it is `[target] message` for every target, including log files, so add a timestamp with `Target::format` if your mobile log files need one. The key-values attached to a record (e.g. the `keyValues` log option in JavaScript) are not printed by the default format.

`Builder::format` and `Builder::clear_format` replace the format. `Builder::timezone_strategy` also replaces it, with `[date][time][level][target] message`, so call `timezone_strategy` before `format`. With the `colored` feature, `Builder::with_colors` replaces it too, and its ANSI escape codes are also written to log files.

### Using your own logger

`Builder::skip_logger` keeps the plugin from installing its logger, for example if you already install one, use `tracing` or register the plugin more than once in tests. The `log` command then forwards JavaScript records to whatever logger is installed. `Builder::split` builds the plugin and returns its logger instead of installing it, so you can combine it with another one.

## Security considerations

- Any webview that is granted `log:default` can write arbitrary messages to your log files and other targets, including text that looks like a record from your Rust code. Treat log content coming from the `webview` target as untrusted. With the default 40 KB `KeepOne` rotation, a webview can also flush older records out of the log file, and with `KeepAll` it can fill the disk.
- The `Webview` target sends every record, including records from third-party crates, to every webview of the app. `attachLogger` and `attachConsole` only need the permission to listen to events (`core:event:default`), not a `log:` permission. Do not enable this target if your app loads remote or untrusted content, or if your logs may contain secrets.

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
