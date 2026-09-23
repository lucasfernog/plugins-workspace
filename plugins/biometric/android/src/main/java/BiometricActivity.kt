// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.biometric

import android.annotation.SuppressLint
import android.app.Activity
import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import android.hardware.biometrics.BiometricManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import androidx.appcompat.app.AppCompatActivity
import androidx.biometric.BiometricPrompt
import java.util.concurrent.Executor

class BiometricActivity : AppCompatActivity() {
    private lateinit var prompt: BiometricPrompt
    private var finished = false
    private var failedAttempts = 0

    @SuppressLint("WrongConstant")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.auth_activity)

        val executor = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            this.mainExecutor
        } else {
            Executor { command: Runnable? ->
                Handler(this.mainLooper).post(
                    command!!
                )
            }
        }

        val builder = BiometricPrompt.PromptInfo.Builder()
        val intent = intent
        var title = intent.getStringExtra(BiometricPlugin.TITLE)
        val subtitle = intent.getStringExtra(BiometricPlugin.SUBTITLE)
        val description = intent.getStringExtra(BiometricPlugin.REASON)
        allowDeviceCredential = false
        // Android docs say we should check if the device is secure before enabling device credential fallback
        val manager = getSystemService(
            Context.KEYGUARD_SERVICE
        ) as KeyguardManager
        if (manager.isDeviceSecure) {
            allowDeviceCredential =
                intent.getBooleanExtra(BiometricPlugin.DEVICE_CREDENTIAL, false)
        }

        if (title.isNullOrEmpty()) {
            title = "Authenticate"
        }

        builder.setTitle(title).setSubtitle(subtitle).setDescription(description)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            var authenticators = BiometricManager.Authenticators.BIOMETRIC_WEAK
            if (allowDeviceCredential) {
                authenticators = authenticators or BiometricManager.Authenticators.DEVICE_CREDENTIAL
            }
            builder.setAllowedAuthenticators(authenticators)
        } else {
            @Suppress("DEPRECATION")
            builder.setDeviceCredentialAllowed(allowDeviceCredential)
        }

        // From the Android docs:
        //  You can't call setNegativeButtonText() and setAllowedAuthenticators(... or DEVICE_CREDENTIAL)
        //  at the same time on a BiometricPrompt.PromptInfo.Builder instance.
        if (!allowDeviceCredential) {
            val negativeButtonText = intent.getStringExtra(BiometricPlugin.CANCEL_TITLE)
            builder.setNegativeButtonText(
                if (negativeButtonText.isNullOrEmpty()) "Cancel" else negativeButtonText
            )
        }
        builder.setConfirmationRequired(
            intent.getBooleanExtra(BiometricPlugin.CONFIRMATION_REQUIRED, true)
        )
        // 0 means no limit: the prompt stays open until the system locks biometry out
        val maxAttempts = intent.getIntExtra(BiometricPlugin.MAX_ATTEMPTS, 0)
        val promptInfo = builder.build()
        prompt = BiometricPrompt(
            this,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationError(
                    errorCode: Int,
                    errorMessage: CharSequence
                ) {
                    super.onAuthenticationError(errorCode, errorMessage)
                    finishActivity(
                        BiometryResultType.ERROR,
                        errorCode,
                        errorMessage as String
                    )
                }

                override fun onAuthenticationFailed() {
                    super.onAuthenticationFailed()
                    // A biometric was presented but not recognized; the prompt stays open for a retry
                    failedAttempts++
                    if (maxAttempts > 0 && failedAttempts >= maxAttempts) {
                        finishActivity(
                            BiometryResultType.FAILURE,
                            0,
                            "Authentication failed after $failedAttempts attempt(s)"
                        )
                        // triggers onAuthenticationError(ERROR_CANCELED), ignored since the activity already finished
                        prompt.cancelAuthentication()
                    }
                }

                override fun onAuthenticationSucceeded(
                    result: BiometricPrompt.AuthenticationResult
                ) {
                    super.onAuthenticationSucceeded(result)
                    finishActivity()
                }
            }
        )
        prompt.authenticate(promptInfo)
    }

    @JvmOverloads
    fun finishActivity(
        resultType: BiometryResultType = BiometryResultType.SUCCESS,
        errorCode: Int = 0,
        errorMessage: String? = ""
    ) {
        // only report the first result (e.g. the cancellation after reaching the max attempts is ignored)
        if (finished) {
            return
        }
        finished = true
        val intent = Intent()
        val prefix = BiometricPlugin.RESULT_EXTRA_PREFIX
        intent
            .putExtra(prefix + BiometricPlugin.RESULT_TYPE, resultType.toString())
            .putExtra(prefix + BiometricPlugin.RESULT_ERROR_CODE, errorCode)
            .putExtra(
                prefix + BiometricPlugin.RESULT_ERROR_MESSAGE,
                errorMessage
            )
        setResult(Activity.RESULT_OK, intent)
        finish()
    }

    companion object {
        var allowDeviceCredential = false
    }
}