![plugin-notification](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/notification/banner.png)

Send message notifications (brief auto-expiring OS window element) to your user. Can also be used with the Notification Web API.

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
tauri-plugin-notification = "2"
```

or, to use the Git sources:

```toml
[dependencies]
tauri-plugin-notification = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-notification
# or
npm add @tauri-apps/plugin-notification
# or
yarn add @tauri-apps/plugin-notification
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Then you need to add the permissions to your capabilities file:

`src-tauri/capabilities/main.json`

```json
{
  ...
  "permissions": [
    ...
    "notification:default"
  ],
  ...
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@tauri-apps/plugin-notification'

async function checkPermission() {
  if (!(await isPermissionGranted())) {
    return (await requestPermission()) === 'granted'
  }
  return true
}

export async function enqueueNotification(title, body) {
  if (!(await checkPermission())) {
    return
  }
  sendNotification({ title, body })
}
```

### Notification with Sound

The `sound` option takes a platform-specific value:

- **macOS**: a system sound name (e.g. `Ping`, `Blow`) or a sound file in the app bundle.
- **Linux**: an XDG sound theme name (e.g. `message-new-instant`).
- **Windows**: one of the built-in toast sounds: `Default`, `IM`, `Mail`, `Reminder`, `SMS`, `Alarm`, `Alarm2`-`Alarm10`, `Call`, `Call2`-`Call10`. Sound files are not supported.
- **Android**: the name of a sound resource in the app's `res/raw` folder. On Android 8 and later the sound is decided by the notification channel, so create a channel with that sound (`createChannel`) and send the notification with its `channelId`.
- **iOS**: the name of a sound file in the app bundle.

```javascript
import { sendNotification } from '@tauri-apps/plugin-notification'
import { platform } from '@tauri-apps/plugin-os'

let sound
switch (platform()) {
  case 'macos':
    sound = 'Ping'
    break
  case 'linux':
    sound = 'message-new-instant'
    break
  case 'windows':
    sound = 'Mail'
    break
  default:
    // mobile: a sound resource bundled with the app
    sound = 'notification'
}

sendNotification({
  title: 'New Message',
  body: 'You have a new message',
  sound
})
```

This example uses [`@tauri-apps/plugin-os`](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/os) to detect the platform.

## Platform notes

- **Desktop** only uses the title, body, icon and sound of a notification. Scheduling, actions, channels, attachments and the other options are ignored, and the mobile-only functions (`registerActionTypes`, `pending`, `cancel`, `cancelAll`, `active`, `removeActive`, `removeAllActive`, `createChannel`, `removeChannel`, `channels`) are not available. The permission is always reported as granted.
- **Windows**: notifications are only shown with the app's name and icon for installed apps. In development they are shown as coming from PowerShell. Windows 7 needs the `windows7-compat` Cargo feature.
- **macOS**: in development notifications are shown as coming from the Terminal.
- **Android**: the plugin adds the `POST_NOTIFICATIONS`, `WAKE_LOCK` and `RECEIVE_BOOT_COMPLETED` permissions to the manifest; `requestPermission()` shows the Android 13+ runtime prompt. Icons must be `res/drawable` resources and sounds `res/raw` resources. Channels (`createChannel`, `channelId`) are Android only. Without the `SCHEDULE_EXACT_ALARM` or `USE_EXACT_ALARM` permission, which the plugin does not declare, scheduled notifications use inexact alarms and may be delivered late; repeating schedules are always inexact. Default values can be set in `tauri.conf.json`:

  ```json
  {
    "plugins": {
      "notification": {
        "icon": "ic_notification",
        "sound": "notification",
        "iconColor": "#ff0000"
      }
    }
  }
  ```

- **iOS**: `requestPermission()` shows the system prompt the first time only; afterwards the user can only change the permission in the Settings app. Attachments (`file://` URLs) are only supported on iOS.
- The plugin injects a script in every webview that replaces `window.Notification`, so the Notification Web API sends notifications through the plugin.

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
