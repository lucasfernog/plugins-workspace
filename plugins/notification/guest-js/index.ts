// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * @module
 */

import {
  invoke,
  type PluginListener,
  addPluginListener
} from '@tauri-apps/api/core'

export type { PermissionState } from '@tauri-apps/api/core'

/**
 * Options to send a notification.
 *
 * @since 2.0.0
 */
interface Options {
  /**
   * The notification identifier to reference this object later. Must be a 32-bit integer.
   */
  id?: number
  /**
   * Identifier of the {@link Channel} that delivers this notification. Android only.
   *
   * If the channel does not exist, the notification won't fire.
   * Make sure the channel exists with {@link channels} and {@link createChannel}.
   */
  channelId?: string
  /**
   * The title displayed on the notification.
   */
  title: string
  /**
   * Optional notification body.
   * */
  body?: string
  /**
   * Schedule this notification to fire on a later time or a fixed interval.
   *
   * Mobile only: desktop notifications are shown immediately.
   */
  schedule?: Schedule
  /**
   * Multiline text.
   * Changes the notification style to big text.
   * Cannot be used with `inboxLines`.
   *
   * Android only.
   */
  largeBody?: string
  /**
   * Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
   *
   * Android only.
   */
  summary?: string
  /**
   * The identifier of an action type registered with {@link registerActionTypes},
   * whose actions are displayed on this notification. Mobile only.
   */
  actionTypeId?: string
  /**
   * Identifier used to group multiple notifications. Mobile only.
   *
   * https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier
   */
  group?: string
  /**
   * Instructs the system that this notification is the summary of a group. Android only.
   */
  groupSummary?: boolean
  /**
   * The sound for the notification.
   *
   * ## Platform-specific behavior:
   *
   * - **macOS**: a system sound name (e.g. `Ping`, `Blow`) or a sound file in the app bundle.
   * - **Linux**: an XDG sound theme name (e.g. `message-new-instant`).
   * - **Windows**: one of the built-in toast sounds: `Default`, `IM`, `Mail`, `Reminder`, `SMS`,
   *   `Alarm`, `Alarm2`-`Alarm10`, `Call`, `Call2`-`Call10`. File paths are not supported.
   * - **Android**: the name of a sound resource in the app's `res/raw` folder. On Android 8+ the
   *   sound is decided by the notification channel, so create a {@link Channel} with this sound.
   * - **iOS**: the name of a sound file in the app bundle.
   */
  sound?: string
  /**
   * List of lines to add to the notification.
   * Changes the notification style to inbox.
   * Cannot be used with `largeBody`.
   *
   * Only supports up to 5 lines. Android only.
   */
  inboxLines?: string[]
  /**
   * Notification icon.
   *
   * On Android the icon must be placed in the app's `res/drawable` folder. Not used on iOS.
   */
  icon?: string
  /**
   * Notification large icon. Android only.
   *
   * The icon must be placed in the app's `res/drawable` folder.
   */
  largeIcon?: string
  /**
   * Icon color, as a hex color string such as `#ff0000`. Android only.
   */
  iconColor?: string
  /**
   * Notification attachments. iOS only.
   */
  attachments?: Attachment[]
  /**
   * Extra payload to store in the notification. Mobile only.
   */
  extra?: Record<string, unknown>
  /**
   * If true, the notification cannot be dismissed by the user. Android only.
   *
   * An application service must manage the dismissal of the notification.
   * It is typically used to indicate a background task that is pending (e.g. a file download)
   * or the user is engaged with (e.g. playing music).
   */
  ongoing?: boolean
  /**
   * Automatically cancel the notification when the user clicks on it. Android only.
   */
  autoCancel?: boolean
  /**
   * Presents the notification without a banner, sound or badge while the app is in the foreground.
   * iOS only; it does not affect notifications delivered while the app is in the background.
   */
  silent?: boolean
  /**
   * How much of the notification is shown on the lock screen. Android only.
   */
  visibility?: Visibility
  /**
   * Sets the number of items this notification represents. Android only.
   */
  number?: number
}

/**
 * The set of date fields a scheduled notification must match to be delivered.
 *
 * Fields that are not set match any value, so the notification fires on every date
 * whose remaining components match.
 */
interface ScheduleInterval {
  /**
   * The year the notification fires on.
   */
  year?: number
  /**
   * The month of the year the notification fires on.
   *
   * The base differs per platform: on Android `0` is January, on iOS `1` is January.
   */
  month?: number
  /**
   * The day of the month the notification fires on.
   */
  day?: number
  /**
   * The day of the week the notification fires on.
   *
   * 1 - Sunday
   * 2 - Monday
   * 3 - Tuesday
   * 4 - Wednesday
   * 5 - Thursday
   * 6 - Friday
   * 7 - Saturday
   */
  weekday?: number
  /**
   * The hour of the day the notification fires on, in the 24-hour clock.
   */
  hour?: number
  /**
   * The minute of the hour the notification fires on.
   */
  minute?: number
  /**
   * The second of the minute the notification fires on.
   */
  second?: number
}

/**
 * The unit of the repeating interval used by {@link Schedule.every}.
 */
enum ScheduleEvery {
  /**
   * The notification repeats every year.
   *
   * On Android a year is approximated as 52 weeks.
   */
  Year = 'year',
  /**
   * The notification repeats every month.
   *
   * On Android a month is approximated as 30 days.
   */
  Month = 'month',
  /**
   * The notification repeats every two weeks.
   */
  TwoWeeks = 'twoWeeks',
  /**
   * The notification repeats every week.
   */
  Week = 'week',
  /**
   * The notification repeats every day.
   */
  Day = 'day',
  /**
   * The notification repeats every hour.
   */
  Hour = 'hour',
  /**
   * The notification repeats every minute.
   */
  Minute = 'minute',
  /**
   * The notification repeats every second.
   *
   * Not supported on iOS, where repeating triggers must be at least a minute apart.
   */
  Second = 'second'
}

/**
 * Defines when a scheduled notification is delivered.
 *
 * Build one with the static {@link Schedule.at}, {@link Schedule.interval} and
 * {@link Schedule.every} helpers, then pass it to the `schedule` option of a notification.
 * Scheduling is only supported on mobile; desktop notifications are always shown immediately.
 *
 * @since 2.0.0
 */
class Schedule {
  /**
   * Set when the notification fires at a fixed date and time.
   */
  at:
    | {
        date: Date
        repeating: boolean
        allowWhileIdle: boolean
      }
    | undefined

  /**
   * Set when the notification fires whenever the current date matches the given fields.
   */
  interval:
    | {
        interval: ScheduleInterval
        allowWhileIdle: boolean
      }
    | undefined

  /**
   * Set when the notification repeats on a fixed interval.
   */
  every:
    | {
        interval: ScheduleEvery
        count: number
        allowWhileIdle: boolean
      }
    | undefined

  /**
   * Creates a schedule that fires the notification at the given date and time.
   *
   * @example
   * ```typescript
   * import { Schedule, sendNotification } from '@tauri-apps/plugin-notification';
   * const schedule = Schedule.at(new Date(Date.now() + 60 * 1000));
   * sendNotification({ title: 'Tauri', body: 'One minute later', schedule });
   * ```
   *
   * @param date The date and time the notification fires at. It must be in the future.
   * @param repeating Whether the notification keeps repeating, using the duration between the moment it is scheduled and `date` as the interval. The interval must be at least one minute on iOS.
   * @param allowWhileIdle Whether the notification is allowed to fire while the device is in low-power idle (Doze) mode. Android only.
   *
   * @returns A schedule that can be assigned to the `schedule` option of a notification.
   */
  static at(date: Date, repeating = false, allowWhileIdle = false): Schedule {
    return {
      at: { date, repeating, allowWhileIdle },
      interval: undefined,
      every: undefined
    }
  }

  /**
   * Creates a schedule that fires the notification whenever the current date matches
   * every field set on the given interval.
   *
   * @example
   * ```typescript
   * import { Schedule, sendNotification } from '@tauri-apps/plugin-notification';
   * // fires every day at 9:00
   * const schedule = Schedule.interval({ hour: 9, minute: 0 });
   * sendNotification({ title: 'Tauri', body: 'Good morning', schedule });
   * ```
   *
   * @param interval The date fields the current date must match for the notification to fire.
   * @param allowWhileIdle Whether the notification is allowed to fire while the device is in low-power idle (Doze) mode. Android only.
   *
   * @returns A schedule that can be assigned to the `schedule` option of a notification.
   */
  static interval(
    interval: ScheduleInterval,
    allowWhileIdle = false
  ): Schedule {
    return {
      at: undefined,
      interval: { interval, allowWhileIdle },
      every: undefined
    }
  }

  /**
   * Creates a schedule that repeatedly fires the notification, once every `count` interval units.
   *
   * @example
   * ```typescript
   * import { Schedule, ScheduleEvery, sendNotification } from '@tauri-apps/plugin-notification';
   * const schedule = Schedule.every(ScheduleEvery.Hour, 2);
   * sendNotification({ title: 'Tauri', body: 'Every two hours', schedule });
   * ```
   *
   * @param kind The unit of the repeating interval.
   * @param count How many interval units elapse between each notification.
   * @param allowWhileIdle Whether the notification is allowed to fire while the device is in low-power idle (Doze) mode. Android only.
   *
   * @returns A schedule that can be assigned to the `schedule` option of a notification.
   */
  static every(
    kind: ScheduleEvery,
    count: number,
    allowWhileIdle = false
  ): Schedule {
    return {
      at: undefined,
      interval: undefined,
      every: { interval: kind, count, allowWhileIdle }
    }
  }
}

/**
 * Attachment of a notification, such as an image. iOS only.
 */
interface Attachment {
  /** Attachment identifier. */
  id: string
  /** Attachment URL. Must be a `file://` URL of a file the app can read. */
  url: string
}

/**
 * A button the user can tap on a notification, belonging to an {@link ActionType}.
 *
 * Only used on mobile. On Android only the identifier, the title and the input flag are used.
 */
interface Action {
  /**
   * The identifier of this action, reported back when the user triggers it.
   */
  id: string
  /**
   * The text displayed on the action button.
   */
  title: string
  /**
   * Whether the device must be unlocked for the action to run. iOS only.
   */
  requiresAuthentication?: boolean
  /**
   * Whether the app is brought to the foreground when the action is triggered. iOS only.
   */
  foreground?: boolean
  /**
   * Whether the action is displayed as destructive, usually in red. iOS only.
   */
  destructive?: boolean
  /**
   * Whether triggering the action lets the user type a text response.
   */
  input?: boolean
  /**
   * The text displayed on the button that submits the text input. iOS only.
   */
  inputButtonTitle?: string
  /**
   * The placeholder displayed on the empty text input field. iOS only.
   */
  inputPlaceholder?: string
}

/**
 * A group of {@link Action}s a notification can display, referenced by the
 * `actionTypeId` option of a notification.
 *
 * Register it with {@link registerActionTypes} before sending a notification that uses it.
 * Only used on mobile.
 */
interface ActionType {
  /**
   * The identifier of this action type
   */
  id: string
  /**
   * The list of associated actions
   */
  actions: Action[]
  /**
   * The placeholder shown instead of the notification body when previews are hidden. iOS only.
   */
  hiddenPreviewsBodyPlaceholder?: string
  /**
   * Whether the app is notified when the user dismisses the notification. iOS only.
   */
  customDismissAction?: boolean
  /**
   * Whether the notification can be displayed in a CarPlay environment. iOS only.
   */
  allowInCarPlay?: boolean
  /**
   * Whether the notification title is shown even when previews are hidden. iOS only.
   */
  hiddenPreviewsShowTitle?: boolean
  /**
   * Whether the notification subtitle is shown even when previews are hidden. iOS only.
   */
  hiddenPreviewsShowSubtitle?: boolean
}

/**
 * A notification that was scheduled and has not been delivered yet.
 *
 * Returned by {@link pending}, which is only supported on mobile.
 */
interface PendingNotification {
  /**
   * The identifier of the notification.
   */
  id: number
  /**
   * The title of the notification, if it was set.
   */
  title?: string
  /**
   * The body of the notification, if it was set.
   */
  body?: string
  /**
   * The schedule that determines when the notification is delivered.
   *
   * On iOS `at` and `every` schedules are both reported as `at`, with the date of the next delivery.
   */
  schedule: Schedule
}

/**
 * A notification that was delivered and is still visible in the notification center.
 *
 * Returned by {@link active}, which is only supported on mobile. Which fields are
 * populated depends on the platform, since Android and iOS expose different
 * information about delivered notifications.
 */
interface ActiveNotification {
  /**
   * The identifier of the notification.
   */
  id: number
  /**
   * The tag the notification was posted with. Android only.
   */
  tag?: string
  /**
   * The title of the notification, if it was set.
   */
  title?: string
  /**
   * The body of the notification, if it was set.
   */
  body?: string
  /**
   * The identifier of the group the notification belongs to. Android only.
   */
  group?: string
  /**
   * Whether the notification is the summary of its group. Android only.
   */
  groupSummary: boolean
  /**
   * The platform extras attached to the notification, as string values. Android only.
   */
  data: Record<string, string>
  /**
   * The extra payload that was stored in the notification.
   */
  extra: Record<string, unknown>
  /**
   * The attachments of the notification. iOS only.
   */
  attachments: Attachment[]
  /**
   * The identifier of the action type the notification was registered with. iOS only.
   */
  actionTypeId?: string
  /**
   * The schedule the notification was delivered with, if it was scheduled.
   */
  schedule?: Schedule
  /**
   * The sound resource name of the notification. iOS only.
   */
  sound?: string
}

/**
 * How much the notifications of a {@link Channel} interrupt the user.
 *
 * It maps to the Android `NotificationManager.IMPORTANCE_*` constants and is only used on Android.
 */
enum Importance {
  /**
   * The notifications are not shown.
   */
  None = 0,
  /**
   * The notifications are only shown in the shade, below the fold, without a status bar icon.
   */
  Min,
  /**
   * The notifications are shown without a sound.
   */
  Low,
  /**
   * The notifications are shown and make a sound.
   *
   * This is the value used when the channel does not define an importance.
   */
  Default,
  /**
   * The notifications are shown, make a sound and pop up as a heads-up notification.
   */
  High
}

/**
 * How much of a notification is shown on the lock screen.
 *
 * It maps to the Android `Notification.VISIBILITY_*` constants and is only used on Android.
 */
enum Visibility {
  /**
   * The notification is not shown on the lock screen at all.
   */
  Secret = -1,
  /**
   * The notification is shown on the lock screen with its sensitive content hidden.
   *
   * This is the value used when the channel does not define a visibility.
   */
  Private,
  /**
   * The notification is shown in full on the lock screen.
   */
  Public
}

/**
 * A notification channel, the category users configure notification behavior on.
 *
 * Notifications reference a channel through their `channelId` option and are not delivered
 * when the channel does not exist. Channels are only supported on Android.
 */
interface Channel {
  /**
   * The identifier of this channel.
   */
  id: string
  /**
   * The user visible name of this channel.
   */
  name: string
  /**
   * The user visible description of this channel.
   */
  description?: string
  /**
   * The name of the sound resource played by the notifications of this channel.
   *
   * The resource must be placed in the app's `res/raw` folder.
   */
  sound?: string
  /**
   * Whether the notifications of this channel blink the device light.
   */
  lights?: boolean
  /**
   * The color of the device light, as a color string such as `#ff0000`.
   */
  lightColor?: string
  /**
   * Whether the notifications of this channel vibrate the device.
   */
  vibration?: boolean
  /**
   * How much the notifications of this channel interrupt the user.
   */
  importance?: Importance
  /**
   * How much of the notifications of this channel is shown on the lock screen.
   */
  visibility?: Visibility
}

/**
 * Checks if the permission to send notifications is granted.
 * @example
 * ```typescript
 * import { isPermissionGranted } from '@tauri-apps/plugin-notification';
 * const permissionGranted = await isPermissionGranted();
 * ```
 *
 * @returns A promise resolving to whether the permission to send notifications is granted.
 *
 * @since 2.0.0
 */
async function isPermissionGranted(): Promise<boolean> {
  if (window.Notification.permission !== 'default') {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await invoke('plugin:notification|is_permission_granted')
}

/**
 * Requests the permission to send notifications.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * ```
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 *
 * @since 2.0.0
 */
async function requestPermission(): Promise<NotificationPermission> {
  return await window.Notification.requestPermission()
}

/**
 * Sends a notification to the user.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * if (permissionGranted) {
 *   sendNotification('Tauri is awesome!');
 *   sendNotification({ title: 'TAURI', body: 'Tauri is awesome!' });
 * }
 * ```
 *
 * @param options The notification content, or the notification title when a string is given.
 *
 * @since 2.0.0
 */
function sendNotification(options: Options | string): void {
  if (typeof options === 'string') {
    new window.Notification(options)
  } else {
    new window.Notification(options.title, options)
  }
}

/**
 * Registers the action types (groups of buttons) notifications can reference through their
 * `actionTypeId` option.
 *
 * Mobile only. On iOS each call replaces the previously registered action types.
 *
 * @example
 * ```typescript
 * import { registerActionTypes } from '@tauri-apps/plugin-notification';
 * await registerActionTypes([{
 *   id: 'tauri',
 *   actions: [{
 *     id: 'my-action',
 *     title: 'Settings'
 *   }]
 * }])
 * ```
 *
 * @param types The action types to register.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function registerActionTypes(types: ActionType[]): Promise<void> {
  await invoke('plugin:notification|register_action_types', { types })
}

/**
 * Retrieves the list of scheduled notifications that have not been delivered yet.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { pending } from '@tauri-apps/plugin-notification';
 * const pendingNotifications = await pending();
 * ```
 *
 * @returns A promise resolving to the list of pending notifications.
 *
 * @since 2.0.0
 */
async function pending(): Promise<PendingNotification[]> {
  return await invoke('plugin:notification|get_pending')
}

/**
 * Cancels the pending notifications with the given list of identifiers.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/plugin-notification';
 * await cancel([-34234, 23432, 4311]);
 * ```
 *
 * @param notifications The identifiers of the pending notifications to cancel.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancel(notifications: number[]): Promise<void> {
  await invoke('plugin:notification|cancel', { notifications })
}

/**
 * Cancels all pending notifications.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { cancelAll } from '@tauri-apps/plugin-notification';
 * await cancelAll();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancelAll(): Promise<void> {
  await invoke('plugin:notification|cancel')
}

/**
 * Retrieves the list of delivered notifications that are still shown in the notification center.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { active } from '@tauri-apps/plugin-notification';
 * const activeNotifications = await active();
 * ```
 *
 * @returns A promise resolving to the list of active notifications.
 *
 * @since 2.0.0
 */
async function active(): Promise<ActiveNotification[]> {
  return await invoke('plugin:notification|get_active')
}

/**
 * Removes the active notifications with the given list of identifiers.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { removeActive } from '@tauri-apps/plugin-notification';
 * await removeActive([{ id: -34234 }, { id: 23432 }, { id: 4311 }])
 * ```
 *
 * @param notifications The active notifications to remove, identified by their id and, on Android, their optional tag.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeActive(
  notifications: Array<{ id: number; tag?: string }>
): Promise<void> {
  await invoke('plugin:notification|remove_active', { notifications })
}

/**
 * Removes all active notifications.
 *
 * Mobile only.
 *
 * @example
 * ```typescript
 * import { removeAllActive } from '@tauri-apps/plugin-notification';
 * await removeAllActive()
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeAllActive(): Promise<void> {
  await invoke('plugin:notification|remove_active')
}

/**
 * Creates a notification channel.
 *
 * Android only: rejects on iOS and is not available on desktop.
 *
 * @example
 * ```typescript
 * import { createChannel, Importance, Visibility } from '@tauri-apps/plugin-notification';
 * await createChannel({
 *   id: 'new-messages',
 *   name: 'New Messages',
 *   lights: true,
 *   vibration: true,
 *   importance: Importance.Default,
 *   visibility: Visibility.Private
 * });
 * ```
 *
 * @param channel The channel to create.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function createChannel(channel: Channel): Promise<void> {
  await invoke('plugin:notification|create_channel', { ...channel })
}

/**
 * Removes the channel with the given identifier.
 *
 * Android only: rejects on iOS and is not available on desktop.
 *
 * @example
 * ```typescript
 * import { removeChannel } from '@tauri-apps/plugin-notification';
 * await removeChannel('new-messages');
 * ```
 *
 * @param id The identifier of the channel to remove.
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeChannel(id: string): Promise<void> {
  await invoke('plugin:notification|delete_channel', { id })
}

/**
 * Retrieves the list of notification channels.
 *
 * Android only: rejects on iOS and is not available on desktop.
 *
 * @example
 * ```typescript
 * import { channels } from '@tauri-apps/plugin-notification';
 * const notificationChannels = await channels();
 * ```
 *
 * @returns A promise resolving to the list of notification channels.
 *
 * @since 2.0.0
 */
async function channels(): Promise<Channel[]> {
  return await invoke('plugin:notification|list_channels')
}

/**
 * Listens to notifications that are delivered while the app is running.
 *
 * Only emitted on mobile.
 *
 * @example
 * ```typescript
 * import { onNotificationReceived } from '@tauri-apps/plugin-notification';
 * const unlisten = await onNotificationReceived((notification) => {
 *   console.log(`received notification: ${notification.title}`);
 * });
 * ```
 *
 * @param cb The closure called with the notification that was delivered.
 *
 * @returns A promise resolving to a listener that can be used to stop listening for the event.
 *
 * @since 2.0.0
 */
async function onNotificationReceived(
  cb: (notification: Options) => void
): Promise<PluginListener> {
  return await addPluginListener('notification', 'notification', cb)
}

/**
 * Listens to the actions the user performs on a notification.
 *
 * Only emitted on mobile, when the user taps a notification or one of the actions of an
 * action type registered with {@link registerActionTypes}.
 *
 * Despite the callback's declared type, the payload is
 * `{ actionId: string, inputValue?: string, notification: object }`: `actionId` is the
 * {@link Action} identifier, or `'tap'` for a tap on the notification itself (`'dismiss'` on iOS when the action type sets `customDismissAction`); `inputValue` is the
 * text typed for an action with `input: true`; `notification` is the notification the action was
 * performed on.
 *
 * @example
 * ```typescript
 * import { onAction } from '@tauri-apps/plugin-notification';
 * const unlisten = await onAction((event) => {
 *   const { actionId, inputValue } = event as unknown as { actionId: string; inputValue?: string };
 *   console.log(`user performed ${actionId}`, inputValue);
 * });
 * ```
 *
 * @param cb The closure called with the action payload.
 *
 * @returns A promise resolving to a listener that can be used to stop listening for the event.
 *
 * @since 2.0.0
 */
async function onAction(
  cb: (notification: Options) => void
): Promise<PluginListener> {
  return await addPluginListener('notification', 'actionPerformed', cb)
}

export type {
  Attachment,
  Options,
  Action,
  ActionType,
  PendingNotification,
  ActiveNotification,
  Channel,
  ScheduleInterval
}

export {
  Importance,
  Visibility,
  sendNotification,
  requestPermission,
  isPermissionGranted,
  registerActionTypes,
  pending,
  cancel,
  cancelAll,
  active,
  removeActive,
  removeAllActive,
  createChannel,
  removeChannel,
  channels,
  onNotificationReceived,
  onAction,
  Schedule,
  ScheduleEvery
}
