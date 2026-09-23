![plugin-stronghold](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/stronghold/banner.png)

Store secrets and keys using the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs) secret management engine.

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
tauri-plugin-stronghold = "2"
# alternatively with Git:
tauri-plugin-stronghold = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

Due to an [upstream bug](https://github.com/tauri-apps/plugins-workspace/issues/2048) we also recommend that you add this to your `Cargo.toml` file:

```toml
[profile.dev.package.scrypt]
opt-level = 3
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: If your JavaScript package manager cannot install packages from git monorepos, you can still use the code by manually copying the [Guest bindings](./guest-js/index.ts) into your source files.

```sh
pnpm add @tauri-apps/plugin-stronghold
# or
npm add @tauri-apps/plugin-stronghold
# or
yarn add @tauri-apps/plugin-stronghold
```

## Usage

First you need to register the core plugin with Tauri. The plugin needs a function that turns the password given to `Stronghold.load` into the 32 bytes key that encrypts the snapshot file.

The recommended setup uses the built-in Argon2 key derivation (the default `kdf` Cargo feature), which stores a random salt in a file the first time it is needed:

`src-tauri/src/lib.rs`

```rust
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let salt_dir = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path");
            // the salt file is written on the first `Stronghold.load`, and its directory
            // might not exist yet on a fresh install
            std::fs::create_dir_all(&salt_dir)?;
            app.handle().plugin(
                tauri_plugin_stronghold::Builder::with_argon2(&salt_dir.join("salt.txt")).build(),
            )?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Keep the salt file: without it, the snapshots can no longer be decrypted.

Alternatively, provide your own password hash function with `Builder::new`. It runs every time the frontend calls `Stronghold.load`, so it must always return the same 32 bytes for the same password. For example with the `rust-argon2` crate:

```rust
tauri::Builder::default()
    .plugin(
        tauri_plugin_stronghold::Builder::new(|password| {
            use argon2::{hash_raw, Config, Variant, Version};

            let config = Config {
                lanes: 4,
                mem_cost: 10_000,
                time_cost: 10,
                variant: Variant::Argon2id,
                version: Version::Version13,
                ..Default::default()
            };

            // Use a random salt generated once per installation and stored next to the
            // snapshot (this is what `Builder::with_argon2` does). A salt hard-coded in
            // the app is shared by every user, which defeats its purpose.
            let salt = load_or_create_salt();

            hash_raw(password.as_ref(), &salt, &config).expect("failed to hash password")
        })
        .build(),
    )
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```typescript
import { Stronghold, Client } from '@tauri-apps/plugin-stronghold'
import { appDataDir, join } from '@tauri-apps/api/path'

const initStronghold = async () => {
  const vaultPath = await join(await appDataDir(), 'vault.hold')
  const vaultPassword = 'The password of the vault'

  const stronghold = await Stronghold.load(vaultPath, vaultPassword)

  const clientName = 'name your client'
  let client: Client
  try {
    client = await stronghold.loadClient(clientName)
  } catch (e) {
    // only create the client if it does not exist yet: creating a client that is
    // already loaded replaces it with an empty one
    if (!String(e).includes('no data present')) {
      throw e
    }
    client = await stronghold.createClient(clientName)
  }

  return { stronghold, client }
}

const { stronghold, client } = await initStronghold()

// Secrets are written to a vault. They can never be read back by the frontend,
// only used through the vault procedures (key derivation, signing, ...).
const vault = client.getVault('my-vault')
await vault.insert('my-secret', Array.from(new TextEncoder().encode('secret')))

// The store is a plain key-value store whose values can be read back, so only use
// it for data that is not secret.
const store = client.getStore()
const key = 'my_key'
await store.insert(key, Array.from(new TextEncoder().encode('Hello, World!')))
const data = await store.get(key)
const value = data ? new TextDecoder().decode(data) : null

// Remove a record from the store
await store.remove(key)

// Changes are only written to the snapshot file on `save()` (or `unload()`)
await stronghold.save()
```

## Permissions

By default no plugin commands are allowed. The `stronghold:default` permission set allows loading and saving strongholds, creating and loading clients, reading and writing store records, writing secrets and running procedures. Removing store records or secrets and unloading a stronghold (`Stronghold.unload()`) must be allowed explicitly:

`src-tauri/capabilities/default.json`

```json
{
  "permissions": [
    "stronghold:default",
    "stronghold:allow-remove-store-record",
    "stronghold:allow-remove-secret",
    "stronghold:allow-destroy"
  ]
}
```

## Security considerations

- The password only protects the snapshot file at rest. Once a snapshot is loaded, every webview with the stronghold permissions can use it by passing its path, without knowing the password, until it is unloaded.
- Snapshot paths come from the frontend and are not checked against any scope (including the file system plugin scope): a webview with `stronghold:default` can create or overwrite a file at any path the app can write to. Only grant the stronghold permissions to windows that load trusted content, and pass absolute paths (relative paths are resolved against the process working directory).
- Loading a snapshot that does not exist yet accepts any password, which becomes the snapshot password on the first save.
- Values in a client store can be read back by the frontend. Keep secrets in a vault, where they can only be used through procedures.

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
