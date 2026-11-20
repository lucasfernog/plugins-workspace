// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit

struct ShareAnchor: Decodable {
  let x: Double
  let y: Double
  var width: Double?
  var height: Double?
}

struct ShareArgs: Decodable {
  var title: String?
  var text: String?
  var url: String?
  var files: [String]?
  var anchor: ShareAnchor?
}

class SharePlugin: Plugin {
  @objc public func share(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ShareArgs.self)

    var items: [Any] = []
    if let text = args.text {
      items.append(text)
    }
    if let url = args.url {
      guard let url = URL(string: url) else {
        invoke.reject("invalid URL: \(url)")
        return
      }
      items.append(url)
    }
    for file in args.files ?? [] {
      let url = file.hasPrefix("file://") ? URL(string: file) : URL(fileURLWithPath: file)
      guard let url = url, FileManager.default.fileExists(atPath: url.path) else {
        invoke.reject("Failed to share file \(file): the file does not exist")
        return
      }
      items.append(url)
    }

    if items.isEmpty {
      invoke.reject("nothing to share: set at least one of text, url and files")
      return
    }

    DispatchQueue.main.async {
      guard let viewController = self.manager.viewController else {
        invoke.reject("Failed to open the share sheet: the app has no view controller")
        return
      }

      let controller = UIActivityViewController(activityItems: items, applicationActivities: nil)
      if let title = args.title {
        // read by the activities that have a subject, such as Mail
        controller.setValue(title, forKey: "subject")
      }

      // on iPad the share sheet is a popover, which crashes without an anchor
      if let popover = controller.popoverPresentationController {
        let view: UIView = viewController.view
        popover.sourceView = view
        if let anchor = args.anchor {
          popover.sourceRect = CGRect(
            x: anchor.x, y: anchor.y, width: anchor.width ?? 0, height: anchor.height ?? 0)
        } else {
          popover.sourceRect = CGRect(x: view.bounds.midX, y: view.bounds.midY, width: 0, height: 0)
          popover.permittedArrowDirections = []
        }
      }

      viewController.present(controller, animated: true) {
        invoke.resolve()
      }
    }
  }
}

@_cdecl("init_plugin_share")
public func initPlugin() -> Plugin {
  return SharePlugin()
}
