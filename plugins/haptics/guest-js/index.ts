// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Trigger haptic feedback on Android and iOS.
 *
 * Unlike most plugins, the functions of this module do not reject when the command fails.
 * They resolve to a {@link Result} instead, which is `{ status: 'error', error }` on failure:
 *
 * ```typescript
 * import { vibrate } from '@tauri-apps/plugin-haptics'
 * const result = await vibrate(300)
 * if (result.status === 'error') {
 *   console.error(result.error)
 * }
 * ```
 *
 * @module
 */

import { commands } from './bindings'
import type {
  ImpactFeedbackStyle,
  NotificationFeedbackType,
  Result,
  Error
} from './bindings'

/**
 * Triggers a vibration for `duration` milliseconds.
 *
 * Has no effect on desktop platforms.
 *
 * @example
 * ```typescript
 * import { vibrate } from '@tauri-apps/plugin-haptics'
 * await vibrate(300)
 * ```
 *
 * @param duration Duration of the vibration, in milliseconds.
 * @returns A promise resolving to `{ status: 'ok', data: null }` on success, or to
 * `{ status: 'error', error }` on failure (for instance when the permission is missing).
 * The promise does **not** reject on these errors, so check `status`.
 * @since 2.0.0
 */
export async function vibrate(duration: number): Promise<Result<null, Error>> {
  return commands.vibrate(duration)
}

/**
 * Triggers an impact-feedback haptic, indicating a collision between user interface elements.
 *
 * On iOS this maps to a `UIImpactFeedbackGenerator` of the given style. On Android, which has
 * no equivalent system API, each style plays a distinct vibration waveform. Has no effect on
 * desktop platforms.
 *
 * @example
 * ```typescript
 * import { impactFeedback } from '@tauri-apps/plugin-haptics'
 * await impactFeedback('medium')
 * ```
 *
 * @param style The style of the impact.
 * @returns A promise resolving to `{ status: 'ok', data: null }` on success, or to
 * `{ status: 'error', error }` on failure (for instance when the permission is missing).
 * The promise does **not** reject on these errors, so check `status`.
 * @since 2.0.0
 */
export async function impactFeedback(
  style: ImpactFeedbackStyle
): Promise<Result<null, Error>> {
  return commands.impactFeedback(style)
}

/**
 * Triggers a notification-feedback haptic, indicating the outcome of a task or action.
 *
 * On iOS this maps to a `UINotificationFeedbackGenerator` of the given type. On Android, which
 * has no equivalent system API, each type plays a distinct vibration waveform. Has no effect on
 * desktop platforms.
 *
 * @example
 * ```typescript
 * import { notificationFeedback } from '@tauri-apps/plugin-haptics'
 * await notificationFeedback('success')
 * ```
 *
 * @param type The outcome to convey.
 * @returns A promise resolving to `{ status: 'ok', data: null }` on success, or to
 * `{ status: 'error', error }` on failure (for instance when the permission is missing).
 * The promise does **not** reject on these errors, so check `status`.
 * @since 2.0.0
 */
export async function notificationFeedback(
  type: NotificationFeedbackType
): Promise<Result<null, Error>> {
  return commands.notificationFeedback(type)
}

/**
 * Triggers a haptic indicating that a selection changed, e.g. the value of a picker control.
 *
 * Has no effect on desktop platforms.
 *
 * @example
 * ```typescript
 * import { selectionFeedback } from '@tauri-apps/plugin-haptics'
 * await selectionFeedback()
 * ```
 *
 * @returns A promise resolving to `{ status: 'ok', data: null }` on success, or to
 * `{ status: 'error', error }` on failure (for instance when the permission is missing).
 * The promise does **not** reject on these errors, so check `status`.
 * @since 2.0.0
 */
export async function selectionFeedback(): Promise<Result<null, Error>> {
  return commands.selectionFeedback()
}

export { ImpactFeedbackStyle, NotificationFeedbackType } from './bindings'

// export { events };
