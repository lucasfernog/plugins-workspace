// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import os.log

@_cdecl("tauri_log")
// `level` must match the `u8` declared on the Rust side of this FFI function.
func log(level: UInt8, message: NSString) {
  switch level {
  case 1: Logger.debug(message as String)
  case 2: Logger.info(message as String)
  case 3: Logger.error(message as String)
  default: break
  }
}
