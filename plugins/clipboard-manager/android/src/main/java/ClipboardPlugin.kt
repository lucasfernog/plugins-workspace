// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.clipboard

import android.app.Activity
import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.core.JsonParser
import com.fasterxml.jackson.core.JsonProcessingException
import com.fasterxml.jackson.databind.DeserializationContext
import com.fasterxml.jackson.databind.JsonDeserializer
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.SerializerProvider
import com.fasterxml.jackson.databind.annotation.JsonDeserialize
import com.fasterxml.jackson.databind.annotation.JsonSerialize
import com.fasterxml.jackson.databind.ser.std.StdSerializer
import java.io.IOException

@InvokeArg
@JsonDeserialize(using = WriteOptionsDeserializer::class)
sealed class WriteOptions {
  @JsonDeserialize
  class PlainText: WriteOptions() {
    lateinit var text: String
    var label: String? = null
  }
}

@JsonSerialize(using = ReadClipDataSerializer::class)
sealed class ReadClipData {
  class PlainText: ReadClipData() {
    lateinit var text: String
  }
}

internal class ReadClipDataSerializer @JvmOverloads constructor(t: Class<ReadClipData>? = null) :
  StdSerializer<ReadClipData>(t) {
  @Throws(IOException::class, JsonProcessingException::class)
  override fun serialize(
    value: ReadClipData, jgen: JsonGenerator, provider: SerializerProvider
  ) {
    jgen.writeStartObject()
    when (value) {
      is ReadClipData.PlainText -> {
        jgen.writeObjectFieldStart("plainText")

        jgen.writeStringField("text", value.text)

        jgen.writeEndObject()
      }
      else -> {
        throw Exception("unimplemented ReadClipData")
      }
    }

    jgen.writeEndObject()
  }
}

internal class WriteOptionsDeserializer: JsonDeserializer<WriteOptions>() {
  override fun deserialize(
    jsonParser: JsonParser,
    deserializationContext: DeserializationContext
  ): WriteOptions {
    val node: JsonNode = jsonParser.codec.readTree(jsonParser)
    node.get("plainText")?.let {
      return jsonParser.codec.treeToValue(it, WriteOptions.PlainText::class.java)
    } ?: run {
      throw Error("unknown write options $node")
    }
  }
}

@TauriPlugin
class ClipboardPlugin(private val activity: Activity) : Plugin(activity) {
  private val manager: ClipboardManager =
    activity.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

  @Command
  @Suppress("MoveVariableDeclarationIntoWhen")
  fun writeText(invoke: Invoke) {
    val args = invoke.parseArgs(WriteOptions::class.java)

    val clipData = when (args) {
      is WriteOptions.PlainText -> {
        ClipData.newPlainText(args.label, args.text)
      } else -> {
        invoke.reject("unimplemented WriteOptions")
        return
      }

    }

    manager.setPrimaryClip(clipData)

    invoke.resolve()
  }

  @Command
  fun readText(invoke: Invoke) {
    // read the clip once: it can change between `hasPrimaryClip()` and `primaryClip`
    val clip = manager.primaryClip
    if (clip == null || clip.itemCount == 0) {
      if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
        invoke.reject("Clipboard is empty or cannot be read: since Android 10 the clipboard can only be read while the app has input focus")
      } else {
        invoke.reject("Clipboard is empty")
      }
      return
    }

    // accept any text type (text/plain, text/html, text/uri-list, ...), not only plain text
    if (clip.description?.hasMimeType("text/*") != true) {
      invoke.reject("Clipboard content is not text")
      return
    }

    val data = ReadClipData.PlainText()
    // `item.text` is null for HTML-only, URI and intent items; coerce them to plain text instead
    data.text = clip.getItemAt(0).coerceToText(activity).toString()
    invoke.resolveObject(data)
  }

  @Command
  fun clear(invoke: Invoke) {
      if (manager.hasPrimaryClip()) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
          manager.clearPrimaryClip()
        } else {
          manager.setPrimaryClip(ClipData.newPlainText("", ""))
        }
      }
      invoke.resolve()
  }
}
