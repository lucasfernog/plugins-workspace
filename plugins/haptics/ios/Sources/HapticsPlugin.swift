// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import AudioToolbox
import CoreHaptics
import SwiftRs
import Tauri
import UIKit
import WebKit

class ImpactFeedbackOptions: Decodable {
  let style: ImpactFeedbackStyle
}

enum ImpactFeedbackStyle: String, Decodable {
  case light, medium, heavy, soft, rigid

  func into() -> UIImpactFeedbackGenerator.FeedbackStyle {
    switch self {
    case .light:
      return .light
    case .medium:
      return .medium
    case .heavy:
      return .heavy
    case .soft:
      return .soft
    case .rigid:
      return .rigid
    }
  }
}

class NotificationFeedbackOptions: Decodable {
  let type: NotificationFeedbackType
}

enum NotificationFeedbackType: String, Decodable {
  case success, warning, error

  func into() -> UINotificationFeedbackGenerator.FeedbackType {
    switch self {
    case .success:
      return .success
    case .warning:
      return .warning
    case .error:
      return .error
    }
  }
}

class VibrateOptions: Decodable {
  // TODO: Array
  let duration: Double
}

class HapticsPlugin: Plugin {
  // A single Core Haptics engine, created on first use and kept for the lifetime of the plugin.
  // Only accessed from the main thread.
  private var engine: CHHapticEngine?
  // The player of the current `vibrate` call, kept so a new call can stop it.
  private var player: CHHapticPatternPlayer?

  //
  // Tauri commands
  //

  @objc public func vibrate(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(VibrateOptions.self)
    DispatchQueue.main.async {
      self.playVibration(duration: args.duration / 1000)
      invoke.resolve()
    }
  }

  private func hapticEngine() throws -> CHHapticEngine {
    if let engine = engine {
      return engine
    }
    let engine = try CHHapticEngine()
    // let the engine stop the haptic hardware while idle; starting a player starts it again
    engine.isAutoShutdownEnabled = true
    engine.resetHandler = { [weak self] in
      // the haptic server was reset: players created before are invalid, restart the engine
      DispatchQueue.main.async {
        guard let self = self else { return }
        self.player = nil
        do {
          try self.engine?.start()
        } catch {
          self.engine = nil
        }
      }
    }
    self.engine = engine
    return engine
  }

  /// Plays a continuous vibration of `duration` seconds, replacing the current one.
  /// A zero duration only stops the current vibration.
  private func playVibration(duration: Double) {
    guard duration > 0 else {
      try? player?.stop(atTime: CHHapticTimeImmediate)
      player = nil
      return
    }

    guard CHHapticEngine.capabilitiesForHardware().supportsHaptics else {
      AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
      return
    }

    do {
      let engine = try hapticEngine()
      try engine.start()

      // like on Android, a new vibration replaces the one currently playing
      try? player?.stop(atTime: CHHapticTimeImmediate)
      player = nil

      // TODO: Make some of this (or all) configurable?
      let intensity: CHHapticEventParameter = CHHapticEventParameter(
        parameterID: .hapticIntensity, value: 1.0)
      let sharpness: CHHapticEventParameter = CHHapticEventParameter(
        parameterID: .hapticSharpness, value: 1.0)
      let continuousEvent = CHHapticEvent(
        eventType: .hapticContinuous,
        parameters: [intensity, sharpness],
        relativeTime: 0.0,
        duration: duration
      )
      let pattern = try CHHapticPattern(events: [continuousEvent], parameters: [])
      let player = try engine.makePlayer(with: pattern)

      try player.start(atTime: CHHapticTimeImmediate)
      self.player = player
    } catch {
      AudioServicesPlayAlertSound(kSystemSoundID_Vibrate)
    }
  }

  @objc public func impactFeedback(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ImpactFeedbackOptions.self)
    let generator = UIImpactFeedbackGenerator(style: args.style.into())
    generator.prepare()
    generator.impactOccurred()

    invoke.resolve()
  }

  @objc public func notificationFeedback(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(NotificationFeedbackOptions.self)
    let generator = UINotificationFeedbackGenerator()
    generator.prepare()
    generator.notificationOccurred(args.type.into())

    invoke.resolve()
  }

  // TODO: Consider breaking this up into Start,Change,End like capacitor
  @objc public func selectionFeedback(_ invoke: Invoke) throws {
    let generator = UISelectionFeedbackGenerator()
    generator.prepare()
    generator.selectionChanged()

    invoke.resolve()
  }
}

@_cdecl("init_plugin_haptics")
func initPlugin() -> Plugin {
  return HapticsPlugin()
}
