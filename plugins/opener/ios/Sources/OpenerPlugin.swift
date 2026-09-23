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

struct OpenPathArgs: Decodable {
  let path: String
}

class OpenerPlugin: Plugin, UIDocumentInteractionControllerDelegate {
  // retained while the preview or "open in" menu is shown
  private var documentController: UIDocumentInteractionController?

  @objc public func open(_ invoke: Invoke) throws {
    do {
      let args = try invoke.parseArgs(OpenArgs.self)
      if let url = URL(string: args.url) {
        if args.with == "inAppBrowser" {
          DispatchQueue.main.async {
            let safariVC = SFSafariViewController(url: url)
            self.manager.viewController?.present(safariVC, animated: true)
          }
        } else {
          if #available(iOS 10, *) {
            UIApplication.shared.open(url, options: [:])
          } else {
            UIApplication.shared.openURL(url)
          }
        }

      }
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }
}

extension OpenerPlugin {
  @objc public func openPath(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(OpenPathArgs.self)
    let url: URL
    if args.path.hasPrefix("file://"), let fileUrl = URL(string: args.path) {
      url = fileUrl
    } else {
      url = URL(fileURLWithPath: args.path)
    }

    guard FileManager.default.fileExists(atPath: url.path) else {
      invoke.reject("Path does not exist: \(args.path)")
      return
    }

    DispatchQueue.main.async {
      guard let viewController = self.manager.viewController else {
        invoke.reject("No view controller to present the file from")
        return
      }

      let controller = UIDocumentInteractionController(url: url)
      controller.delegate = self
      self.documentController = controller

      // iOS has no "default application" for a local file: preview it with Quick Look
      // (which offers sharing / opening in another app), or fall back to the "open in" menu.
      if controller.presentPreview(animated: true)
        || controller.presentOpenInMenu(
          from: viewController.view.bounds, in: viewController.view, animated: true)
      {
        invoke.resolve()
      } else {
        self.documentController = nil
        invoke.reject("No application can open this file")
      }
    }
  }

  public func documentInteractionControllerViewControllerForPreview(
    _ controller: UIDocumentInteractionController
  ) -> UIViewController {
    return self.manager.viewController ?? UIViewController()
  }

  public func documentInteractionControllerDidEndPreview(
    _ controller: UIDocumentInteractionController
  ) {
    self.documentController = nil
  }

  public func documentInteractionControllerDidDismissOpenInMenu(
    _ controller: UIDocumentInteractionController
  ) {
    self.documentController = nil
  }
}

@_cdecl("init_plugin_opener")
func initPlugin() -> Plugin {
  return OpenerPlugin()
}
