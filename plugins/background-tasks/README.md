![background-tasks](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/background-tasks/banner.png)

Schedule background tasks written in Rust on Android and iOS.

Tasks are async Rust functions. The system runs them while the app is in the background, through [WorkManager](https://developer.android.com/develop/background-work/background-tasks/persistent) on Android and [BGTaskScheduler](https://developer.apple.com/documentation/backgroundtasks/bgtaskscheduler) on iOS. The system decides when a task actually runs: a schedule is a request, not a guarantee.

| Platform | Supported |
| -------- | --------- |
| Linux    | x         |
| Windows  | x         |
| macOS    | x         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.90**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-background-tasks = "2.0.0"
# alternatively with Git:
tauri-plugin-background-tasks = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

<!-- Add the branch for installations using git! -->

```sh
pnpm add @tauri-apps/plugin-background-tasks
# or
npm add @tauri-apps/plugin-background-tasks
# or
yarn add @tauri-apps/plugin-background-tasks
```

## Setting up

### iOS

Every task name must be declared in the app's `Info.plist`, together with the background modes the tasks use: `fetch` for `refresh` tasks and `processing` for `processing` tasks.

`src-tauri/Info.ios.plist`

```xml
<key>BGTaskSchedulerPermittedIdentifiers</key>
<array>
  <string>com.example.app.sync</string>
</array>
<key>UIBackgroundModes</key>
<array>
  <string>fetch</string>
  <string>processing</string>
</array>
```

The plugin registers a launch handler for every identifier in that list, so don't register the same identifiers with `BGTaskScheduler` yourself.

Background tasks don't run on the simulator, and not at all when the user disabled Background App Refresh. To trigger one on a device, pause the app in Xcode and run `e -l objc -- (void)[[BGTaskScheduler sharedScheduler] _simulateLaunchForTaskWithIdentifier:@"com.example.app.sync"]` in the debugger.

### Android

No manifest changes are needed. Call `export_tasks!` in your library crate, as shown below: it exports the entry points WorkManager uses to reach your tasks.

To run a task while testing, use the Background Task Inspector of Android Studio, or `adb shell cmd jobscheduler run -f <package> <job-id>` with a job id from `adb shell dumpsys jobscheduler`.

## Usage

Write the tasks, export them and register the plugin:

`src-tauri/src/lib.rs`

```rust
use std::time::Duration;
use tauri_plugin_background_tasks::{
    BackgroundTasksExt, Outcome, TaskContext, TaskRequest, Tasks,
};

async fn sync(ctx: TaskContext) -> Outcome {
    for item in pending_items() {
        // the system can stop a task at any time
        if ctx.is_cancelled() {
            return Outcome::Retry;
        }
        upload(item).await;
    }
    Outcome::Success
}

fn tasks() -> Tasks {
    Tasks::new().task("com.example.app.sync", sync)
}

// lets Android run the tasks when the app is not running
tauri_plugin_background_tasks::export_tasks!(tasks);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_background_tasks::init(tasks()))
        .setup(|app| {
            app.background_tasks().schedule(
                TaskRequest::new("com.example.app.sync")
                    .interval(Duration::from_secs(60 * 60))
                    .requires_network(true),
            )?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

A handler returns `Outcome::Success`, `Outcome::Retry` or `Outcome::Failure`. It can also return `()` or a `Result`, whose error is logged and fails the task.

Tasks can also be scheduled from JavaScript. Add the required permissions in the project:

`src-tauri/capabilities/default.json`

```json
  "permissions": [
    "background-tasks:default",
    "background-tasks:allow-schedule",
    "background-tasks:allow-cancel"
  ]
```

Afterwards the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import {
  schedule,
  cancel,
  getScheduled
} from '@tauri-apps/plugin-background-tasks'

await schedule({
  name: 'com.example.app.sync',
  input: { full: true },
  delay: 5 * 60 * 1000,
  requiresNetwork: true
})
console.log(await getScheduled())
await cancel('com.example.app.sync')
```

## Where a task runs

- **iOS** launches the whole app in the background to run a task, so `ctx.app_handle()` returns the app handle.
- **Android** runs the task in the app's process. If the app is running, `ctx.app_handle()` returns its handle. Otherwise WorkManager starts the process without an activity: the Tauri app and its `setup` hook do not run, and `ctx.app_handle()` returns `None`. Write handlers that work without app state, and avoid crates that need the Android activity or a JNI context set up by Tauri.

## Platform differences

| Option                                | Android                                | iOS                                                                   |
| ------------------------------------- | -------------------------------------- | --------------------------------------------------------------------- |
| `name`                                | Any string                             | Must be listed in `BGTaskSchedulerPermittedIdentifiers`               |
| `input`                               | Up to about 10 KB                      | Stored in `UserDefaults`                                              |
| `interval`                            | At least 15 minutes                    | The task is submitted again each time it runs; the system picks times |
| `requiresNetwork`, `requiresCharging` | Supported                              | Only for `processing` tasks                                           |
| `kind`                                | Ignored, a task can run for 10 minutes | `refresh` gets about 30 seconds, `processing` several minutes         |
| `Outcome::Retry`                      | Retried with WorkManager's backoff     | The request is submitted again                                        |

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
