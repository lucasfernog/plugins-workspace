// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import Security
import SwiftRs
import Tauri
import UIKit
import WebKit

class KeyArgs: Decodable {
  let key: String
}

class SetItemArgs: Decodable {
  let key: String
  let value: String
}

struct GetItemResponse: Encodable {
  // omitted when nil, which the Rust side reads as "no item"
  let value: String?
}

/// Items are stored in the Keychain as generic passwords under the app's bundle identifier.
class SecureStoragePlugin: Plugin {
  private var service: String {
    Bundle.main.bundleIdentifier ?? "tauri-plugin-secure-storage"
  }

  private func query(_ key: String) -> [String: Any] {
    return [
      kSecClass as String: kSecClassGenericPassword,
      kSecAttrService as String: service,
      kSecAttrAccount as String: key,
    ]
  }

  private func message(_ status: OSStatus) -> String {
    let description = SecCopyErrorMessageString(status, nil) as String? ?? "unknown error"
    return "\(description) (OSStatus \(status))"
  }

  @objc public func getItem(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(KeyArgs.self)

    var query = self.query(args.key)
    query[kSecReturnData as String] = true
    query[kSecMatchLimit as String] = kSecMatchLimitOne

    var item: CFTypeRef?
    let status = SecItemCopyMatching(query as CFDictionary, &item)
    switch status {
    case errSecSuccess:
      guard let data = item as? Data, let value = String(data: data, encoding: .utf8) else {
        invoke.reject("the stored value is not valid UTF-8")
        return
      }
      invoke.resolve(GetItemResponse(value: value))
    case errSecItemNotFound:
      invoke.resolve(GetItemResponse(value: nil))
    default:
      invoke.reject("failed to read item: \(message(status))")
    }
  }

  @objc public func setItem(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SetItemArgs.self)
    let data = Data(args.value.utf8)

    var status = SecItemUpdate(
      query(args.key) as CFDictionary,
      [kSecValueData as String: data] as CFDictionary
    )
    if status == errSecItemNotFound {
      var attributes = query(args.key)
      attributes[kSecValueData as String] = data
      // readable while the device is unlocked or was unlocked once since boot (so
      // background work keeps working), and never synced or restored to another device
      attributes[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
      status = SecItemAdd(attributes as CFDictionary, nil)
    }

    if status == errSecSuccess {
      invoke.resolve()
    } else {
      invoke.reject("failed to store item: \(message(status))")
    }
  }

  @objc public func removeItem(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(KeyArgs.self)

    let status = SecItemDelete(query(args.key) as CFDictionary)
    if status == errSecSuccess || status == errSecItemNotFound {
      invoke.resolve()
    } else {
      invoke.reject("failed to remove item: \(message(status))")
    }
  }
}

@_cdecl("init_plugin_secure_storage")
func initPlugin() -> Plugin {
  return SecureStoragePlugin()
}
