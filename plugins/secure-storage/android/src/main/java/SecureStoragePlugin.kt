// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.securestorage

import android.app.Activity
import android.content.Context
import android.content.SharedPreferences
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val ANDROID_KEYSTORE = "AndroidKeyStore"
private const val KEY_ALIAS = "tauri-plugin-secure-storage"
private const val PREFERENCES_NAME = "tauri-plugin-secure-storage"
private const val TRANSFORMATION = "AES/GCM/NoPadding"
private const val IV_LENGTH = 12
private const val TAG_LENGTH_BITS = 128

@InvokeArg
class KeyArgs {
    lateinit var key: String
}

@InvokeArg
class SetItemArgs {
    lateinit var key: String
    lateinit var value: String
}

/**
 * Values are encrypted with an AES-256-GCM key that is generated in, and never leaves,
 * the Android Keystore. The ciphertexts are kept in a private SharedPreferences file.
 */
@TauriPlugin
class SecureStoragePlugin(private val activity: Activity) : Plugin(activity) {
    private val preferences: SharedPreferences by lazy {
        activity.applicationContext.getSharedPreferences(PREFERENCES_NAME, Context.MODE_PRIVATE)
    }

    @Command
    fun getItem(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(KeyArgs::class.java)
            val ret = JSObject()
            // a missing `value` is read as "no item" by the Rust side
            read(args.key)?.let { ret.put("value", it) }
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject(e.message ?: "failed to read item", e)
        }
    }

    @Command
    fun setItem(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SetItemArgs::class.java)
            write(args.key, args.value)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "failed to store item", e)
        }
    }

    @Command
    fun removeItem(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(KeyArgs::class.java)
            if (!preferences.edit().remove(args.key).commit()) {
                throw IllegalStateException("failed to persist the removal")
            }
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject(e.message ?: "failed to remove item", e)
        }
    }

    @Synchronized
    private fun read(key: String): String? {
        val stored = preferences.getString(key, null) ?: return null
        return try {
            val data = Base64.decode(stored, Base64.NO_WRAP)
            // the secret key is gone when the preferences were restored from a backup
            val secretKey = secretKey(create = false) ?: return null
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(
                Cipher.DECRYPT_MODE,
                secretKey,
                GCMParameterSpec(TAG_LENGTH_BITS, data, 0, IV_LENGTH)
            )
            cipher.updateAAD(key.toByteArray(Charsets.UTF_8))
            String(cipher.doFinal(data, IV_LENGTH, data.size - IV_LENGTH), Charsets.UTF_8)
        } catch (e: GeneralSecurityException) {
            // encrypted with a key that no longer exists (backup restored on another
            // install) or tampered with: the value is unrecoverable
            Logger.warn("secure-storage: could not decrypt \"$key\", treating it as missing: $e")
            null
        } catch (e: IllegalArgumentException) {
            Logger.warn("secure-storage: \"$key\" is malformed, treating it as missing: $e")
            null
        }
    }

    @Synchronized
    private fun write(key: String, value: String) {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        // the Keystore provider generates a fresh random IV for every encryption
        cipher.init(Cipher.ENCRYPT_MODE, secretKey(create = true))
        // binds the ciphertext to its key so entries cannot be swapped in the file
        cipher.updateAAD(key.toByteArray(Charsets.UTF_8))
        val ciphertext = cipher.doFinal(value.toByteArray(Charsets.UTF_8))
        val stored = Base64.encodeToString(cipher.iv + ciphertext, Base64.NO_WRAP)
        if (!preferences.edit().putString(key, stored).commit()) {
            throw IllegalStateException("failed to persist the item")
        }
    }

    private fun secretKey(create: Boolean): SecretKey? {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        (keyStore.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        if (!create) {
            return null
        }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build()
        )
        return generator.generateKey()
    }
}
