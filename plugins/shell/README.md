![plugin-shell](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/shell/banner.png)

Access the system shell. Allows you to spawn child processes and manage files and URLs using their default application.

| Platform | Supported                                                        |
| -------- | ---------------------------------------------------------------- |
| Linux    | ✓                                                                |
| Windows  | ✓                                                                |
| macOS    | ✓                                                                |
| Android  | Partial: spawning processes works, see the note on `open`        |
| iOS      | Partial: iOS apps cannot spawn processes, see the note on `open` |

> [!NOTE]
> The `open` API of this plugin (JavaScript `open()` and Rust `Shell::open`) is deprecated since 2.1.0.
> Use [`tauri-plugin-opener`](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/opener) instead.
> On Android and iOS only the Rust `Shell::open` can open URLs; the JavaScript `open()` fails there.

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
tauri-plugin-shell = "2.0.0"
# alternatively with Git:
tauri-plugin-shell = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-shell
# or
npm add @tauri-apps/plugin-shell
# or
yarn add @tauri-apps/plugin-shell
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { Command } from '@tauri-apps/plugin-shell'
// `run-git-commit` is the `name` of a scope entry, see below
const output = await Command.create('run-git-commit', [
  'commit',
  '-m',
  'the-commit-message'
]).execute()
console.log(output.code, output.stdout, output.stderr)
```

## Permissions

By default no program can be run from JavaScript: `shell:default` only allows the deprecated `open` API.
To run a program, grant `shell:allow-execute` (for `Command.execute()`) and/or `shell:allow-spawn` (for `Command.spawn()`,
plus `shell:allow-stdin-write` and `shell:allow-kill` for `Child.write()` and `Child.kill()`) with a scope that lists the
programs and arguments the webview may use. Each of these permissions has its own scope.

`src-tauri/capabilities/default.json`

```json
{
  "permissions": [
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "run-git-commit",
          "cmd": "git",
          "args": ["commit", "-m", { "validator": "\\S+" }]
        }
      ]
    }
  ]
}
```

Each scope entry has:

- `name`: the name passed to `Command.create()` (or `Command.sidecar()`).
- `cmd`: the program to run. It can start with a path variable such as `$APPDATA`. Not used for sidecars.
- `args`: `true` allows any arguments, `false` (the default when omitted) allows none, and a list describes each position:
  a string is a fixed argument and `{ "validator": "<regex>" }` an argument that must match the regex. Validators must
  match the whole argument (they are wrapped in `^...$`) unless `"raw": true` is set.
- `sidecar`: `true` to run a binary bundled with `bundle > externalBin` in `tauri.conf.json` (see
  [Embedding External Binaries](https://v2.tauri.app/develop/sidecar/)); `name` must then be that `externalBin` entry.

The webview can also set the `env` and `cwd` options of any program it is allowed to run, and the scope does not restrict
them. Variables such as `PATH` or `LD_PRELOAD` change which program runs or what code it loads, so only allow programs to
content you trust.

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
