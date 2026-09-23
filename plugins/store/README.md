![plugin-store](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/store/banner.png)

Simple, persistent key-value store.

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
tauri-plugin-store = "2"
# alternatively with Git:
tauri-plugin-store = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-store
# or
npm add @tauri-apps/plugin-store
# or
yarn add @tauri-apps/plugin-store
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```typescript
import { Store } from '@tauri-apps/plugin-store'

const store = await Store.load('settings.json')

await store.set('some-key', { value: 5 })

const val = await store.get<{ value: number }>('some-key')

if (val) {
  console.log(val)
} else {
  console.log('val is null')
}
```

### Persisting Values

Modifications made to the store are automatically saved by default

You can manually save a store with:

```javascript
await store.save()
```

A `Store` is loaded when it is created with `Store.load(path)` (or `load(path)`), and a `LazyStore` is loaded the first time one of its methods is called.
You can also load a `LazyStore` ahead of time with:

```javascript
await store.init()
```

To re-read a store's file after it was changed on disk, use:

```javascript
await store.reload()
```

### LazyStore

There's also a high level API `LazyStore` which only loads the store on first access, note that the options will be ignored if a `Store` with that path has already been created

```typescript
import { LazyStore } from '@tauri-apps/plugin-store'

const store = new LazyStore('settings.json')
```

### Options

```typescript
import { load } from '@tauri-apps/plugin-store'

const store = await load('settings.json', {
  // initial values, the on-disk state is merged on top of them; `reset()` restores them
  defaults: { theme: 'dark' },
  // debounce in milliseconds, `false` disables auto save, `true` or omitted means 100ms, `0` saves on every change
  autoSave: 500,
  // true: use the on-disk state as is instead of merging it into the defaults
  overrideDefaults: false,
  // true: ignore any loaded store and the on-disk state, and start from the defaults
  createNew: false
  // serializeFnName / deserializeFnName: names of functions registered with
  // `tauri_plugin_store::Builder::register_serialize_fn` / `register_deserialize_fn` on the Rust side
})
```

A store is shared by every caller that loads the same path: when it is already loaded, `load` returns that instance and ignores the options.
Every loaded store is also saved when the app exits, even if auto save is disabled.

### Change events

```typescript
const unlisten = await store.onKeyChange<string>('theme', (theme) => {
  console.log('theme changed to', theme)
})
```

`onChange` and `onKeyChange` receive the changes made to the store from any window and from Rust. `reload()` does not emit change events.

## Usage from Rust

You can also create `Store` instances directly in Rust:

```rust
use tauri_plugin_store::StoreExt;
use serde_json::json;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // This loads the store from disk
            let store = app.store("app_data.json")?;

            // Note that values must be serde_json::Value instances,
            // otherwise, they will not be compatible with the JavaScript bindings.
            store.set("a".to_string(), json!("b"));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Interoperability

The store created from both Rust side and JavaScript side are stored in the app's resource table and can be accessed by both sides, you can access it by using the same path, with `getStore` and `LazyStore` in the JavaScript side and `get_store` and `store` in the Rust side

Stores are shared by the whole app: every window gets the same store for the same path, and closing it (`store.close()`) closes it for every window. An `Arc<Store>` held in Rust keeps working after that, but it is no longer the store that the next `load` returns.

## File location and security

Store paths are resolved relative to the app data directory (`BaseDirectory::AppData`, see [`resolve_store_path`](https://docs.rs/tauri-plugin-store/latest/tauri_plugin_store/fn.resolve_store_path.html)).

> [!CAUTION]
> Store paths are not scoped: an absolute path replaces the app data directory and `..` components can leave it, also from the frontend.
> Together with `store:default`, which allows every store command, this lets any frontend code read and overwrite any JSON file the app can access.
> Only grant the store permissions to windows that load trusted content.

If a store file exists but cannot be read or deserialized, the store starts from its defaults and the file is overwritten on the next save.

## Permissions

`store:default` allows every store command: `load`, `get_store`, `set`, `get`, `has`, `delete`, `clear`, `reset`, `keys`, `values`, `entries`, `length`, `reload` and `save`.
To restrict a window, grant the individual `store:allow-*` permissions instead, e.g. `store:allow-load` and `store:allow-get` for read-only access.
Closing a store uses `core:resource:allow-close`, which `core:default` includes.

`src-tauri/capabilities/default.json`

```json
{
  "permissions": ["core:default", "store:default"]
}
```

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
