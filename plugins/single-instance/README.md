![tauri-plugin-single-instance](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/single-instance/banner.png)

Ensure a single instance of your tauri app is running.

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
tauri-plugin-single-instance = "2.0.0"
# alternatively with Git:
tauri-plugin-single-instance = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // the plugin is only available on desktop
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
            // focus the main window of the running instance
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

`argv` are the second instance's command line arguments (the first one is the executable path) and `cwd` is its working directory.

Note that currently, plugins run in the order they were added in to the builder, so make sure that this plugin is registered first. Register it on the builder as shown above, not with `app.handle().plugin()` inside `setup`: by then the windows from `tauri.conf.json` already exist, so a second instance would briefly show them.

### Things to know

- **Treat the callback arguments as untrusted input.** Any process running as the same user can send arguments to the running instance through the plugin's IPC channel (a window message on Windows, D-Bus on Linux, a Unix socket on macOS). Validate them before acting on them, and don't forward them unfiltered to your frontend.
- The callback runs on the main thread on Windows, on a D-Bus executor thread on Linux and on an async runtime worker on macOS. Keep it short, and use `AppHandle::run_on_main_thread` for work that needs the main thread.
- On Windows, a second instance running at a lower integrity level than the running one (for example not elevated while the running instance is elevated) can't reach it: its arguments are dropped and it exits.
- The crate is empty on Android and iOS, so gate the registration with `#[cfg(desktop)]`.
- If you terminate the process with `std::process::exit` or similar, which skips `RunEvent::Exit`, call `tauri_plugin_single_instance::destroy(app)` first.

## Cargo features

- `semver`: lets instances whose versions are SemVer-incompatible (e.g. `1.x` and `2.x`) run side by side, while compatible versions are still limited to one instance.
- `deep-link`: forwards the second instance's arguments to [`tauri-plugin-deep-link`](../deep-link) before your callback runs, so deep links opened while the app is running reach the running instance. Register the deep-link plugin as well.

```toml
[target.'cfg(any(target_os = "macos", windows, target_os = "linux"))'.dependencies]
tauri-plugin-single-instance = { version = "2", features = ["deep-link"] }
```

## Usage with Flatpak/Snap

On Linux the plugin owns the D-Bus name `<identifier>.SingleInstance` on the session bus, where `<identifier>` is the `identifier` from `tauri.conf.json` (with the `semver` feature a version suffix such as `_1_x_x` is appended). Flatpak and Snap only let the app own names under its app ID, so if your Tauri identifier doesn't match the package ID, pick the base name with `Builder::dbus_id`:

```rust
builder = builder.plugin(
    tauri_plugin_single_instance::Builder::new()
        .callback(|app, argv, cwd| { /* ... */ })
        // registers `com.mycompany.myapp.SingleInstance`
        .dbus_id("com.mycompany.myapp")
        .build(),
);
```

Plugin versions before 2.4.0 used `org.<identifier with . and - replaced by _>.SingleInstance`; update the name in your Flatpak/Snap manifests accordingly.

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
