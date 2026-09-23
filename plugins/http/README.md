![plugin-http](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/http/banner.png)

Access the HTTP client written in Rust.

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
tauri-plugin-http = "2.0.0"
# alternatively with Git:
tauri-plugin-http = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-http
# or
npm add @tauri-apps/plugin-http
# or
yarn add @tauri-apps/plugin-http
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { fetch } from '@tauri-apps/plugin-http'
const response = await fetch('http://localhost:3003/users/2', {
  method: 'GET',
  // Tauri-specific client option, in milliseconds
  connectTimeout: 30_000
})
```

## Permissions and scope

The `http:default` permission allows all the commands `fetch` uses, but **no URL**: every request fails with
`url not allowed on the configured scope` until you allow the URLs your frontend may request, in the scope of
the permission in your capability:

`src-tauri/capabilities/default.json`

```json
{
  "permissions": [
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://*.tauri.app" }, "http://localhost:3003"],
      "deny": [{ "url": "https://private.tauri.app" }]
    }
  ]
}
```

- Entries are [URL patterns](https://urlpattern.spec.whatwg.org/) (not globs), given as strings or as `{ "url": "..." }` objects.
  The scheme and the port must match: `https://*.tauri.app` does not match `http://test.tauri.app`, nor `https://tauri.app`
  itself.
- When the path is empty or `/`, it matches any path, and the query and fragment always match when the pattern does not
  set them: `https://tauri.app` allows `https://tauri.app/any/path?query`.
- `deny` entries take precedence over `allow` entries.
- `data:` URLs are not subject to the scope.

### Redirects

By default, only the URL requested by the frontend is checked against the scope: redirects are followed to any URL,
so an allowed server that redirects elsewhere (an open redirect, `localhost`, an internal host, a cloud metadata
endpoint...) gives the frontend access to URLs the scope does not allow. Set the `scopeRedirects` option (since
2.7.0) to check every redirect target against the scope too:

`src-tauri/tauri.conf.json`

```json
{
  "plugins": {
    "http": {
      "scopeRedirects": true
    }
  }
}
```

### Proxies

The `proxy` option of `fetch` is not checked against the scope: a frontend allowed to request a single URL can route
it through any host and port it picks. Keep this in mind when granting `http` permissions to remote or less trusted
content.

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
