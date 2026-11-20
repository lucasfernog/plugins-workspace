// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.share

import android.app.Activity
import android.content.ClipData
import android.content.Intent
import android.net.Uri
import android.webkit.MimeTypeMap
import androidx.core.content.FileProvider
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class ShareArgs {
    var title: String? = null
    var text: String? = null
    var url: String? = null
    var files: Array<String> = arrayOf()
}

@TauriPlugin
class SharePlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun share(invoke: Invoke) {
        val args = invoke.parseArgs(ShareArgs::class.java)

        val uris = ArrayList<Uri>()
        for (file in args.files) {
            try {
                uris.add(fileUri(file))
            } catch (e: Exception) {
                invoke.reject("Failed to share file $file: ${e.message}")
                return
            }
        }

        // ACTION_SEND has a single text field, so the URL goes after the text
        val text = listOfNotNull(args.text, args.url).joinToString("\n")
        if (text.isEmpty() && uris.isEmpty()) {
            invoke.reject("nothing to share: set at least one of text, url and files")
            return
        }

        val intent = Intent(if (uris.size > 1) Intent.ACTION_SEND_MULTIPLE else Intent.ACTION_SEND)
        if (text.isNotEmpty()) {
            intent.putExtra(Intent.EXTRA_TEXT, text)
        }
        args.title?.let {
            intent.putExtra(Intent.EXTRA_SUBJECT, it)
            // shown in the share sheet's content preview
            intent.putExtra(Intent.EXTRA_TITLE, it)
        }

        if (uris.isEmpty()) {
            intent.type = "text/plain"
        } else {
            intent.type = commonMimeType(uris)
            if (uris.size == 1) {
                intent.putExtra(Intent.EXTRA_STREAM, uris[0])
            } else {
                intent.putParcelableArrayListExtra(Intent.EXTRA_STREAM, uris)
            }
            // the read grant only covers the URIs in the intent's data and ClipData, not its extras
            val clip = ClipData.newUri(activity.contentResolver, args.title ?: "", uris[0])
            for (uri in uris.drop(1)) {
                clip.addItem(ClipData.Item(uri))
            }
            intent.clipData = clip
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }

        try {
            activity.startActivity(Intent.createChooser(intent, args.title))
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject("Failed to open the share sheet: ${e.message}")
        }
    }

    private fun fileUri(file: String): Uri {
        if (file.startsWith("content://")) {
            return Uri.parse(file)
        }
        val path = if (file.startsWith("file://")) Uri.parse(file).path ?: file else file
        val target = File(path)
        if (!target.isFile) {
            throw IllegalArgumentException("the file does not exist")
        }
        return try {
            FileProvider.getUriForFile(
                activity,
                "${activity.packageName}.share.fileprovider",
                target
            )
        } catch (e: IllegalArgumentException) {
            // FileProvider only serves the directories listed in tauri_share_file_paths.xml
            throw IllegalArgumentException(
                "only files in the app's data and cache directories can be shared"
            )
        }
    }

    private fun mimeType(uri: Uri): String? {
        activity.contentResolver.getType(uri)?.let { return it }
        val extension = uri.lastPathSegment?.substringAfterLast('.', "") ?: ""
        return MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension.lowercase())
    }

    // the most specific MIME type that covers all the files, e.g. image/* for a PNG and a JPEG
    private fun commonMimeType(uris: List<Uri>): String {
        val types = uris.map { mimeType(it) ?: return "*/*" }.distinct()
        if (types.size == 1) {
            return types[0]
        }
        val groups = types.map { it.substringBefore('/') }.distinct()
        return if (groups.size == 1) "${groups[0]}/*" else "*/*"
    }
}
