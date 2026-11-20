// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Schedule the background tasks the app registered in Rust. The system runs them
 * while the app is in the background, on Android and iOS.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * How much runtime a task needs. Only iOS distinguishes the two kinds.
 *
 * - `refresh`: a short task that refreshes the app's content. On iOS it gets around
 *   30 seconds and ignores `requiresNetwork` and `requiresCharging`.
 * - `processing`: a longer task, such as a cleanup or a large upload. On iOS it gets
 *   several minutes, usually while the device is idle.
 *
 * @since 2.0.0
 */
export type TaskKind = 'refresh' | 'processing'

/**
 * A request to run one of the tasks the app registered in Rust.
 *
 * @since 2.0.0
 */
export interface TaskRequest {
  /**
   * The name the task handler was registered with. On iOS it must be listed in the
   * `BGTaskSchedulerPermittedIdentifiers` array of the app's `Info.plist`.
   */
  name: string
  /**
   * JSON value handed to the task handler. Android limits it to about 10 KB.
   */
  input?: unknown
  /**
   * Minimum time to wait before the task first runs, in milliseconds.
   */
  delay?: number
  /**
   * Run the task repeatedly, at most once per this many milliseconds.
   * Android raises anything below 15 minutes to 15 minutes.
   */
  interval?: number
  /**
   * Only run the task while the device has a network connection.
   */
  requiresNetwork?: boolean
  /**
   * Only run the task while the device is charging.
   */
  requiresCharging?: boolean
  /**
   * How much runtime the task needs. Defaults to `refresh`.
   */
  kind?: TaskKind
}

/**
 * A task that is waiting to run.
 *
 * @since 2.0.0
 */
export interface ScheduledTask {
  /**
   * The name the task was scheduled with.
   */
  name: string
}

/**
 * Schedules a task, replacing the pending request with the same name.
 * The system decides when the task actually runs.
 *
 * @example
 * ```typescript
 * import { schedule } from '@tauri-apps/plugin-background-tasks';
 * await schedule({
 *   name: 'com.example.app.sync',
 *   input: { full: true },
 *   interval: 60 * 60 * 1000,
 *   requiresNetwork: true
 * });
 * ```
 *
 * @since 2.0.0
 */
export async function schedule(request: TaskRequest): Promise<void> {
  await invoke('plugin:background-tasks|schedule', { request })
}

/**
 * Cancels the pending request of the task with the given name.
 * On Android this also stops the task if it is running.
 *
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/plugin-background-tasks';
 * await cancel('com.example.app.sync');
 * ```
 *
 * @since 2.0.0
 */
export async function cancel(name: string): Promise<void> {
  await invoke('plugin:background-tasks|cancel', { name })
}

/**
 * Cancels every pending request scheduled through this plugin.
 *
 * @example
 * ```typescript
 * import { cancelAll } from '@tauri-apps/plugin-background-tasks';
 * await cancelAll();
 * ```
 *
 * @since 2.0.0
 */
export async function cancelAll(): Promise<void> {
  await invoke('plugin:background-tasks|cancel_all')
}

/**
 * Returns the tasks that are waiting to run.
 *
 * @example
 * ```typescript
 * import { getScheduled } from '@tauri-apps/plugin-background-tasks';
 * const tasks = await getScheduled();
 * console.log(tasks.map((task) => task.name));
 * ```
 *
 * @since 2.0.0
 */
export async function getScheduled(): Promise<ScheduledTask[]> {
  return await invoke('plugin:background-tasks|get_scheduled')
}
