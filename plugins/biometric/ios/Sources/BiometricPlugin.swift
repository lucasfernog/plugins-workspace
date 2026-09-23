// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import LocalAuthentication
import SwiftRs
import Tauri
import UIKit
import WebKit

class BiometricStatus {
  let available: Bool
  let biometryType: LABiometryType
  let errorReason: String?
  let errorCode: String?

  init(available: Bool, biometryType: LABiometryType, errorReason: String?, errorCode: String?) {
    self.available = available
    self.biometryType = biometryType
    self.errorReason = errorReason
    self.errorCode = errorCode
  }
}

struct AuthOptions: Decodable {
  let reason: String
  var allowDeviceCredential: Bool?
  var fallbackTitle: String?
  var cancelTitle: String?
}

class BiometricPlugin: Plugin {
  let authenticationErrorCodeMap: [Int: String] = [
    0: "",
    LAError.appCancel.rawValue: "appCancel",
    LAError.authenticationFailed.rawValue: "authenticationFailed",
    LAError.invalidContext.rawValue: "invalidContext",
    LAError.notInteractive.rawValue: "notInteractive",
    LAError.passcodeNotSet.rawValue: "passcodeNotSet",
    LAError.systemCancel.rawValue: "systemCancel",
    LAError.userCancel.rawValue: "userCancel",
    LAError.userFallback.rawValue: "userFallback",
    LAError.biometryLockout.rawValue: "biometryLockout",
    LAError.biometryNotAvailable.rawValue: "biometryNotAvailable",
    LAError.biometryNotEnrolled.rawValue: "biometryNotEnrolled",
  ]

  /// Evaluates the current biometry state. Called on every `status` and `authenticate` call
  /// (instead of once when the plugin loads) so enrolment and lockout changes made while the app
  /// runs are picked up.
  func currentStatus() -> BiometricStatus {
    let context = LAContext()
    var error: NSError?
    var available = context.canEvaluatePolicy(
      .deviceOwnerAuthenticationWithBiometrics, error: &error)
    var reason: String? = nil
    var errorCode: String? = nil

    if available && context.biometryType == .faceID {
      let entry = Bundle.main.infoDictionary?["NSFaceIDUsageDescription"] as? String

      if entry == nil || entry?.count == 0 {
        available = false
        reason = "NSFaceIDUsageDescription is not in the app Info.plist"
        errorCode = authenticationErrorCodeMap[LAError.biometryNotAvailable.rawValue] ?? ""
      }
    } else if !available, let error = error {
      reason = error.localizedDescription
      if let failureReason = error.localizedFailureReason {
        reason = "\(reason ?? ""): \(failureReason)"
      }
      errorCode =
        authenticationErrorCodeMap[error.code] ?? authenticationErrorCodeMap[
          LAError.biometryNotAvailable.rawValue] ?? ""
    }

    return BiometricStatus(
      available: available,
      biometryType: context.biometryType,
      errorReason: reason,
      errorCode: errorCode
    )
  }

  @objc func status(_ invoke: Invoke) {
    let status = self.currentStatus()
    if status.available {
      invoke.resolve([
        "isAvailable": status.available,
        "biometryType": status.biometryType.rawValue,
      ])
    } else {
      invoke.resolve([
        "isAvailable": status.available,
        "biometryType": status.biometryType.rawValue,
        "error": status.errorReason ?? "",
        "errorCode": status.errorCode ?? "",
      ])
    }
  }

  @objc func authenticate(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(AuthOptions.self)

    let allowDeviceCredential = args.allowDeviceCredential ?? false

    // only evaluate the status when it matters: with device credentials allowed the OS falls back to the passcode
    if !allowDeviceCredential {
      let status = self.currentStatus()
      guard status.available else {
        // Biometry unavailable, fallback disabled
        invoke.reject(
          status.errorReason ?? "",
          code: status.errorCode ?? ""
        )
        return
      }
    }

    let context = LAContext()
    context.localizedFallbackTitle = args.fallbackTitle
    context.localizedCancelTitle = args.cancelTitle
    context.touchIDAuthenticationAllowableReuseDuration = 0

    // force system default fallback title if an empty string is provided (the OS hides the fallback button in this case)
    if allowDeviceCredential,
      let fallbackTitle = context.localizedFallbackTitle,
      fallbackTitle.isEmpty
    {
      context.localizedFallbackTitle = nil
    }

    context.evaluatePolicy(
      allowDeviceCredential
        ? .deviceOwnerAuthentication : .deviceOwnerAuthenticationWithBiometrics,
      localizedReason: args.reason
    ) { success, error in
      if success {
        invoke.resolve()
      } else {
        if let policyError = error as? LAError {
          let code = self.authenticationErrorCodeMap[policyError.code.rawValue]
          invoke.reject(policyError.localizedDescription, code: code)
        } else {
          invoke.reject(
            "Unknown error",
            code: self.authenticationErrorCodeMap[LAError.authenticationFailed.rawValue]
          )
        }
      }
    }

  }
}

@_cdecl("init_plugin_biometric")
func initPlugin() -> Plugin {
  return BiometricPlugin()
}
