// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.backgroundtasks

import android.content.Context
import androidx.work.Worker
import androidx.work.WorkerParameters
import app.tauri.Logger

/** Runs one Rust task handler on a WorkManager background thread. */
class BackgroundTaskWorker(context: Context, params: WorkerParameters) : Worker(context, params) {
    companion object {
        private const val TAG = "BackgroundTasks"

        const val KEY_NAME = "name"
        const val KEY_INPUT = "input"
    }

    private val runId = id.toString()

    override fun doWork(): Result {
        val name = inputData.getString(KEY_NAME) ?: return Result.failure()
        val input = inputData.getString(KEY_INPUT) ?: "null"

        if (!TaskRunner.load(applicationContext)) {
            return Result.failure()
        }

        return try {
            when (TaskRunner.run(runId, name, input)) {
                TaskRunner.SUCCESS -> Result.success()
                TaskRunner.RETRY -> Result.retry()
                else -> Result.failure()
            }
        } catch (e: UnsatisfiedLinkError) {
            Logger.error(
                TAG,
                "the app does not export its background tasks, call tauri_plugin_background_tasks::export_tasks! in its library crate",
                e
            )
            Result.failure()
        }
    }

    override fun onStopped() {
        try {
            TaskRunner.cancel(runId)
        } catch (_: UnsatisfiedLinkError) {
            // the library was not loaded, so the task is not running
        }
    }
}
