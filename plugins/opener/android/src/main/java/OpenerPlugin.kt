// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.opener

import android.app.Activity
import android.content.Intent
import androidx.browser.customtabs.CustomTabsIntent
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import androidx.core.net.toUri
import androidx.core.content.FileProvider
import app.tauri.annotation.InvokeArg
import java.io.File

@InvokeArg
class OpenArgs {
  lateinit var url: String
  var with: String? = null
}

@InvokeArg
class OpenPathArgs {
  lateinit var path: String
}

@TauriPlugin
class OpenerPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun open(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(OpenArgs::class.java)

            if (args.with == "inAppBrowser") {
                val builder = CustomTabsIntent.Builder()
                val intent = builder.build()
                intent.launchUrl(activity, args.url.toUri())
            } else {
                val intent = Intent(Intent.ACTION_VIEW, args.url.toUri())
                intent.setFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
                activity.applicationContext?.startActivity(intent)
            }
            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message)
        }
    }

    @Command
    fun openPath(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(OpenPathArgs::class.java)

            val uri = if (args.path.startsWith("content://")) {
                args.path.toUri()
            } else {
                val file = if (args.path.startsWith("file://")) {
                    File(args.path.toUri().path ?: args.path)
                } else {
                    File(args.path)
                }
                if (!file.exists()) {
                    invoke.reject("Path does not exist: ${args.path}")
                    return
                }
                // `file://` URIs cannot be shared with other apps (FileUriExposedException)
                FileProvider.getUriForFile(
                    activity,
                    "${activity.packageName}.opener.fileprovider",
                    file
                )
            }

            val intent = Intent(Intent.ACTION_VIEW)
            intent.setDataAndType(uri, activity.contentResolver.getType(uri) ?: "*/*")
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
            activity.startActivity(intent)
            invoke.resolve()
        } catch (ex: Exception) {
            invoke.reject(ex.message)
        }
    }
}
