![plugin-persisted-scope](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/persisted-scope/banner.png)

Save filesystem and asset scopes and restore them when the app is reopened.

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
tauri-plugin-persisted-scope = "2.0.0"
# alternatively with Git:
tauri-plugin-persisted-scope = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

## Usage

First you need to register the core plugin with Tauri. The [`fs`](../fs) plugin must be registered **before** this plugin, otherwise the filesystem scope is not saved or restored (a warning is only printed in debug builds):

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards the plugin will automatically save and restore the `fs` plugin scope. Only runtime changes to the scope are persisted, for example paths picked with the dialog plugin, dropped onto a window, or allowed from Rust with `app.fs_scope()`. Scopes defined in capability files are not affected.

To also persist the [asset protocol](https://v2.tauri.app/security/asset-protocol/) scope, enable the `protocol-asset` feature:

```toml
[dependencies]
tauri-plugin-persisted-scope = { version = "2", features = ["protocol-asset"] }
```

The state is stored in the app data directory, in `.persisted-scope` (filesystem scope) and `.persisted-scope-asset` (asset protocol scope). Delete these files to reset all persisted grants.

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
