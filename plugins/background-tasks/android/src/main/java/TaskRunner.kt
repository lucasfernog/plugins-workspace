// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.backgroundtasks

import android.content.Context
import app.tauri.Logger

/**
 * Bridge to the task handlers written in Rust.
 *
 * The native methods are exported from the app's library by the
 * `tauri_plugin_background_tasks::export_tasks!` macro.
 */
object TaskRunner {
    private const val TAG = "BackgroundTasks"

    const val SUCCESS = 0
    const val RETRY = 1
    const val FAILURE = 2

    private var loaded = false

    /**
     * Makes sure the app's Rust library is loaded.
     *
     * When WorkManager starts the process only to run a task no activity is created, so
     * nothing has loaded the library yet. The app's generated `Rust` object loads it when
     * initialized. It lives in the same package as the launcher activity.
     */
    @Synchronized
    fun load(context: Context): Boolean {
        if (loaded) {
            return true
        }

        val activity = context.packageManager
            .getLaunchIntentForPackage(context.packageName)?.component?.className
        if (activity == null) {
            Logger.error(TAG, "could not find the launcher activity to load the Rust library", null)
            return false
        }

        val candidates = listOf("${activity.substringBeforeLast('.')}.Rust", activity)
        for (candidate in candidates) {
            try {
                Class.forName(candidate, true, context.classLoader)
                loaded = true
                return true
            } catch (_: ClassNotFoundException) {
                // try the next candidate
            } catch (e: Throwable) {
                Logger.error(TAG, "failed to load the Rust library through $candidate", e)
                return false
            }
        }

        Logger.error(TAG, "could not find the class that loads the Rust library", null)
        return false
    }

    /** Runs the task handler registered as [name] and blocks until it returns a result code. */
    @JvmStatic
    external fun run(runId: String, name: String, input: String): Int

    /** Tells the handler of the given run to stop. */
    @JvmStatic
    external fun cancel(runId: String)
}
