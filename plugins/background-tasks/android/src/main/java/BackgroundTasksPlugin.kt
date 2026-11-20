// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.backgroundtasks

import android.app.Activity
import androidx.work.Constraints
import androidx.work.Data
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.ExistingWorkPolicy
import androidx.work.NetworkType
import androidx.work.OneTimeWorkRequest
import androidx.work.PeriodicWorkRequest
import androidx.work.WorkManager
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.concurrent.TimeUnit

@InvokeArg
class ScheduleArgs {
    lateinit var name: String
    var input: String? = null
    var delay: Long? = null
    var interval: Long? = null
    var requiresNetwork: Boolean = false
    var requiresCharging: Boolean = false
    // only iOS distinguishes task kinds
    var kind: String? = null
}

@InvokeArg
class CancelArgs {
    lateinit var name: String
}

// every request scheduled by the plugin has this tag
private const val TAG_ALL = "app.tauri.backgroundtasks"
// and this one followed by the task name
private const val TAG_NAME_PREFIX = "app.tauri.backgroundtasks.name:"
// keeps the plugin's unique work names apart from the app's own
private const val UNIQUE_NAME_PREFIX = "app.tauri.backgroundtasks:"

@TauriPlugin
class BackgroundTasksPlugin(private val activity: Activity) : Plugin(activity) {
    private val workManager: WorkManager
        get() = WorkManager.getInstance(activity.applicationContext)

    @Command
    fun schedule(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(ScheduleArgs::class.java)

            val constraints = Constraints.Builder()
                .setRequiredNetworkType(
                    if (args.requiresNetwork) NetworkType.CONNECTED else NetworkType.NOT_REQUIRED
                )
                .setRequiresCharging(args.requiresCharging)
                .build()
            // throws if the input is larger than WorkManager allows (about 10 KB)
            val data = Data.Builder()
                .putString(BackgroundTaskWorker.KEY_NAME, args.name)
                .putString(BackgroundTaskWorker.KEY_INPUT, args.input ?: "null")
                .build()
            val uniqueName = UNIQUE_NAME_PREFIX + args.name
            val delay = args.delay ?: 0

            val interval = args.interval
            if (interval != null) {
                val request = PeriodicWorkRequest.Builder(
                    BackgroundTaskWorker::class.java,
                    interval,
                    TimeUnit.MILLISECONDS
                )
                    .setConstraints(constraints)
                    .setInputData(data)
                    .setInitialDelay(delay, TimeUnit.MILLISECONDS)
                    .addTag(TAG_ALL)
                    .addTag(TAG_NAME_PREFIX + args.name)
                    .build()
                workManager.enqueueUniquePeriodicWork(
                    uniqueName,
                    ExistingPeriodicWorkPolicy.CANCEL_AND_REENQUEUE,
                    request
                )
            } else {
                val request = OneTimeWorkRequest.Builder(BackgroundTaskWorker::class.java)
                    .setConstraints(constraints)
                    .setInputData(data)
                    .setInitialDelay(delay, TimeUnit.MILLISECONDS)
                    .addTag(TAG_ALL)
                    .addTag(TAG_NAME_PREFIX + args.name)
                    .build()
                workManager.enqueueUniqueWork(uniqueName, ExistingWorkPolicy.REPLACE, request)
            }

            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message, e)
        }
    }

    @Command
    fun cancel(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CancelArgs::class.java)
            workManager.cancelUniqueWork(UNIQUE_NAME_PREFIX + args.name)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message, e)
        }
    }

    @Command
    fun cancelAll(invoke: Invoke) {
        try {
            workManager.cancelAllWorkByTag(TAG_ALL)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message, e)
        }
    }

    @Command
    fun getScheduled(invoke: Invoke) {
        val infos = workManager.getWorkInfosByTag(TAG_ALL)
        // reading the result blocks on WorkManager's database
        Thread {
            try {
                val tasks = JSArray()
                for (info in infos.get()) {
                    if (info.state.isFinished) {
                        continue
                    }
                    val name = info.tags.firstOrNull { it.startsWith(TAG_NAME_PREFIX) }
                        ?.removePrefix(TAG_NAME_PREFIX) ?: continue
                    val task = JSObject()
                    task.put("name", name)
                    tasks.put(task)
                }
                val result = JSObject()
                result.put("tasks", tasks)
                invoke.resolve(result)
            } catch (e: Exception) {
                invoke.reject(e.message, e)
            }
        }.start()
    }
}
