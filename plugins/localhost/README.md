![plugin-localhost](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/localhost/banner.png)

Expose your apps assets through a localhost server instead of the default custom protocol.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

> Note: This plugins brings considerable security risks and you should only use it if you know what your are doing. If in doubt, use the default custom protocol implementation.

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
tauri-plugin-localhost = "2"
# alternatively with Git:
tauri-plugin-localhost = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg(not(dev))]
use tauri::{ipc::CapabilityBuilder, Manager, Url};
use tauri::{WebviewUrl, WebviewWindowBuilder};
#[cfg(not(dev))]
use tauri_plugin_localhost::LocalhostExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Port 0 lets the operating system pick a free port. Bind to 127.0.0.1 explicitly:
        // `localhost` only binds one address family, leaving the other one to other processes.
        .plugin(
            tauri_plugin_localhost::Builder::new(0)
                .host("127.0.0.1")
                .build(),
        )
        .setup(|app| {
            // In `tauri dev` mode you usually use your dev server.
            #[cfg(dev)]
            let url = WebviewUrl::App(std::path::PathBuf::from("/"));

            #[cfg(not(dev))]
            let url = {
                // The address the server is actually bound to, including the picked port.
                let addr = app.localhost_addr().expect("localhost plugin not registered");
                let url: Url = format!("http://{addr}").parse().unwrap();

                // The page is now served from a remote origin, which has no IPC access by
                // default. This grants it to the `main` window when it loads this URL.
                app.add_capability(
                    CapabilityBuilder::new("localhost")
                        .remote(url.to_string())
                        .window("main"),
                )?;

                WebviewUrl::External(url)
            };

            // This requires you to remove the window from tauri.conf.json
            WebviewWindowBuilder::new(app, "main".to_string(), url)
                .title("Localhost Example")
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

If the server can't bind to its address, plugin setup fails and the app doesn't start.

The server also runs during `tauri dev`. If `devUrl` is set and `frontendDist` is a directory, it serves the files from `frontendDist` on disk, so build your frontend first if you want to load it through the server in dev mode.

Anything granted to the `localhost` capability is available to whatever page is loaded from that URL. The server has no authentication, so any local process can read the assets it serves.

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
