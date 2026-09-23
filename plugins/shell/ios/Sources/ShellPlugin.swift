// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

import SwiftRs
import Tauri
import UIKit
import WebKit

class ShellPlugin: Plugin {

    @objc public func open(_ invoke: Invoke) throws {
        do {
            let urlString = try invoke.parseArgs(String.self)
            guard let url = URL(string: urlString) else {
                invoke.reject("Invalid URL: \(urlString)")
                return
            }
            // Commands run on a background queue, and UIKit must be used from the main thread.
            // The result of the open is not waited for: the Rust `Shell::open` can be called
            // from the main thread and waits for this command, so it would never resolve.
            DispatchQueue.main.async {
                UIApplication.shared.open(url, options: [:], completionHandler: nil)
            }
            invoke.resolve()
        } catch {
            invoke.reject(error.localizedDescription)
        }
    }
}

@_cdecl("init_plugin_shell")
func initPlugin() -> Plugin {
  return ShellPlugin()
}
