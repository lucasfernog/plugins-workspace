// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import SafariServices
import SwiftRs
import Tauri
import UIKit
import WebKit

struct OpenArgs: Decodable {
  let url: String
  let with: String?
}

class OpenerPlugin: Plugin {
  @objc public func open(_ invoke: Invoke) throws {
    do {
      let args = try invoke.parseArgs(OpenArgs.self)
      guard let url = URL(string: args.url) else {
        invoke.reject("Invalid URL: \(args.url)")
        return
      }

      // SFSafariViewController only supports http(s) URLs (it raises an exception otherwise),
      // so other schemes are opened with their default app
      let scheme = url.scheme?.lowercased()
      if args.with == "inAppBrowser" && (scheme == "http" || scheme == "https") {
        DispatchQueue.main.async {
          guard let viewController = self.manager.viewController else {
            invoke.reject("No view controller to present the in-app browser from")
            return
          }
          viewController.present(SFSafariViewController(url: url), animated: true)
          invoke.resolve()
        }
      } else {
        DispatchQueue.main.async {
          UIApplication.shared.open(url, options: [:]) { opened in
            if opened {
              invoke.resolve()
            } else {
              invoke.reject("Failed to open URL: \(args.url)")
            }
          }
        }
      }
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }
}

@_cdecl("init_plugin_opener")
func initPlugin() -> Plugin {
  return OpenerPlugin()
}
