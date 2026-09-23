![plugin-upload](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/upload/banner.png)

Upload files from disk to a remote server over HTTP.
Download files from a remote HTTP server to disk.

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

Install the plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-upload = "2.0.0"
# alternatively with Git:
tauri-plugin-upload = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-upload
# or
npm add @tauri-apps/plugin-upload
# or
yarn add @tauri-apps/plugin-upload
```

## Usage

First you need to register the plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_upload::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings.

### Upload

`upload` sends the contents of a file as the request body and resolves with the response body as text:

```javascript
import { upload, HttpMethod } from '@tauri-apps/plugin-upload'
import { appDataDir, join } from '@tauri-apps/api/path'

const filePath = await join(await appDataDir(), 'report.pdf')

// Upload with the default POST method
const response = await upload(
  'https://example.com/file-upload',
  filePath,
  ({ progressTotal, total }) =>
    console.log(`Uploaded ${progressTotal} of ${total} bytes`), // a callback that will be called with the upload progress
  { Authorization: 'Bearer <token>' } // optional headers to send with the request
)

// Upload with a specific HTTP method
await upload(
  'https://example.com/file-upload',
  filePath,
  ({ progressTotal, total }) =>
    console.log(`Uploaded ${progressTotal} of ${total} bytes`),
  { Authorization: 'Bearer <token>' },
  HttpMethod.Put // Use the HttpMethod enum - supports POST (default), PUT and PATCH
)
```

The `Content-Length` header is set from the file size. Upload progress is reported as the file is read into the request body, so it can reach 100% before the server has received everything and replied.

### Download

`download` streams the response body into a file:

```javascript
import { download } from '@tauri-apps/plugin-upload'
import { appDataDir, join } from '@tauri-apps/api/path'

await download(
  'https://example.com/file-download-link',
  await join(await appDataDir(), 'file.zip'), // the parent directory must already exist
  ({ progressTotal, total }) =>
    console.log(`Downloaded ${progressTotal} of ${total} bytes`), // a callback that will be called with the download progress
  { Authorization: 'Bearer <token>' } // optional headers to send with the request
)
```

The request is a `GET`, unless the optional fifth argument `body` is given: then a `POST` request with that string as its body is sent.
`total` is `0` when the server does not send a `Content-Length` header or compresses the response.

### Progress and errors

The progress callback receives `progress` (the size of the last chunk, not the cumulative count), `progressTotal` (the bytes transferred so far), `total` and `transferSpeed` (bytes per second, recalculated about every 500 ms).

Both functions reject when the file cannot be read or written, when the request fails, or when the server replies with a non-2xx status (`request failed with status code <code>: <response body>`). A transfer cannot be cancelled once it has started, and there is no timeout.

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
