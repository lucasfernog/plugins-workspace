![plugin-updater](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/updater/banner.png)

In-app updates for Tauri applications.

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
# you can add the dependencies on the `[dependencies]` section if you do not target mobile
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
tauri-plugin-updater = "2.0.0"
```

Or, with Git:

```toml
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
tauri-plugin-updater = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-updater
# or
npm add @tauri-apps/plugin-updater
# or
yarn add @tauri-apps/plugin-updater
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
const update = await check()
if (update) {
  await update.downloadAndInstall()
  // Relaunch the app on macOS and Linux to run the newly install version
  await relaunch()
}
```

`relaunch` comes from the [process plugin](https://v2.tauri.app/plugin/process/), which must be installed and registered too, with the `process:allow-restart` permission granted. On Windows the app exits when the installer starts (and the installer relaunches it by default, see `restartAfterInstall`), so `relaunch` is never reached there.

Note that for these APIs to work you have to properly configure the updater first and generate updater artifacts. Please refer to the [guide on our website](https://v2.tauri.app/plugin/updater/) for this.

### Configuration

The plugin is configured under `plugins > updater` in `tauri.conf.json`, and the updater artifacts are created when `bundle > createUpdaterArtifacts` is `true`:

| Key                                  | Description                                                                                                                                                                                                     |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pubkey`                             | **Required.** The content of the public key generated with `tauri signer generate` (not a path), used to verify the update signatures.                                                                          |
| `endpoints`                          | The URLs the update manifest is fetched from, tried in order. They can contain the `{{current_version}}`, `{{target}}`, `{{arch}}` and `{{bundle_type}}` variables. Must use `https` in release builds.         |
| `windows.installMode`                | `passive` (default), `basicUi` or `quiet`.                                                                                                                                                                      |
| `windows.installerArgs`              | Additional arguments passed to the NSIS or MSI installer.                                                                                                                                                       |
| `requireSignedVersion`               | Reject an update whose signature does not carry the version announced by the endpoint. Recommended, it stops a tampered manifest from serving an older signed release. Re-sign old releases before enabling it. |
| `allowDowngrades`                    | Install any release whose version differs from the current one, instead of only newer ones. Ignored when a custom version comparator is set.                                                                    |
| `dangerousInsecureTransportProtocol` | Allow non-`https` endpoints in release builds.                                                                                                                                                                  |
| `dangerousAcceptInvalidCerts`        | Accept invalid TLS certificates. Never use this in production.                                                                                                                                                  |
| `dangerousAcceptInvalidHostnames`    | Accept TLS certificates for other hostnames. Never use this in production.                                                                                                                                      |

An endpoint answers with `204 No Content` when there is no update, or with a JSON manifest: either the "dynamic" format (`version`, `url`, `signature`, and optionally `notes` and `pub_date`) or the "static" format, where `url` and `signature` are listed per target in a `platforms` object. See the [guide](https://v2.tauri.app/plugin/updater/) for the details.

### Permissions

The `updater:default` permission set allows all the commands: `check`, `download`, `install` and `download_and_install`. Note that it lets the frontend pick the `proxy` and the `target` of the update check. Grant the individual `updater:allow-*` permissions instead if you only need some of them.

### Platform notes

- **Windows:** `install()` never returns when it succeeds: the app exits to let the installer run.
- **macOS:** when the app's directory is not writable (e.g. `/Applications` for a non-admin user), the user is asked for an administrator password.
- **Linux:** AppImage updates replace the AppImage in place. `.deb` and `.rpm` updates are installed with `dpkg -i` / `rpm -U`, as root through `pkexec`, then a `zenity`/`kdialog` password prompt with `sudo`, then plain `sudo`. `dpkg -i` does not install missing dependencies.
- The whole update is downloaded into memory before it is verified and installed.

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
