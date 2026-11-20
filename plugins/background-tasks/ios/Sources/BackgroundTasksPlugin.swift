// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import BackgroundTasks
import SwiftRs
import Tauri
import UIKit
import WebKit

struct ScheduleArgs: Codable {
  let name: String
  var input: String?
  var delay: Double?
  var interval: Double?
  var requiresNetwork: Bool?
  var requiresCharging: Bool?
  var kind: String?
}

struct CancelArgs: Decodable {
  let name: String
}

struct SetHandlerArgs: Decodable {
  let channel: Channel
}

struct CompleteArgs: Decodable {
  let runId: String
  let outcome: String
}

/// Sent to Rust when the system launches a task ("run") or asks it to stop ("expire").
struct TaskEvent: Encodable {
  let event: String
  let runId: String
  var name: String?
  var input: String?
}

struct ScheduledTask: Encodable {
  let name: String
}

struct ScheduledResponse: Encodable {
  let tasks: [ScheduledTask]
}

/// A task launched by the system whose Rust handler has not completed yet.
private class Run {
  /// Nil once the task expired and was reported to the system as completed.
  var task: BGTask?
  let args: ScheduleArgs?

  init(task: BGTask, args: ScheduleArgs?) {
    self.task = task
    self.args = args
  }
}

/// BGTaskRequest carries no data, so the requests are kept here by task name to hand
/// the input to the handler and to submit periodic and retried tasks again.
private let requestsKey = "app.tauri.background-tasks.requests"

class BackgroundTasksPlugin: Plugin {
  private let lock = NSLock()
  private var channel: Channel?
  /// Events for tasks launched before Rust registered its channel.
  private var pendingEvents: [TaskEvent] = []
  private var runs: [String: Run] = [:]

  override init() {
    super.init()

    // Launch handlers must be registered before the app finishes launching. Tauri
    // creates the plugins before it starts the app's run loop, so this is early enough.
    let identifiers =
      Bundle.main.object(forInfoDictionaryKey: "BGTaskSchedulerPermittedIdentifiers")
      as? [String] ?? []
    for identifier in identifiers {
      let registered = BGTaskScheduler.shared.register(
        forTaskWithIdentifier: identifier, using: nil
      ) { [weak self] task in
        if let plugin = self {
          plugin.launch(task)
        } else {
          task.setTaskCompleted(success: false)
        }
      }
      if !registered {
        Logger.error("failed to register the background task \(identifier)")
      }
    }
  }

  //
  // Stored requests
  //

  private func storedRequests() -> [String: String] {
    return UserDefaults.standard.dictionary(forKey: requestsKey) as? [String: String] ?? [:]
  }

  private func storedRequest(_ name: String) -> ScheduleArgs? {
    guard let json = storedRequests()[name]?.data(using: .utf8) else {
      return nil
    }
    return try? JSONDecoder().decode(ScheduleArgs.self, from: json)
  }

  private func store(_ args: ScheduleArgs) throws {
    let json = String(decoding: try JSONEncoder().encode(args), as: UTF8.self)
    lock.lock()
    defer { lock.unlock() }
    var requests = storedRequests()
    requests[args.name] = json
    UserDefaults.standard.set(requests, forKey: requestsKey)
  }

  private func removeStoredRequest(_ name: String) {
    lock.lock()
    defer { lock.unlock() }
    var requests = storedRequests()
    requests.removeValue(forKey: name)
    UserDefaults.standard.set(requests, forKey: requestsKey)
  }

  //
  // Running tasks
  //

  private func submit(_ args: ScheduleArgs, delay: Double?) throws {
    let request: BGTaskRequest
    if args.kind == "processing" {
      let processing = BGProcessingTaskRequest(identifier: args.name)
      processing.requiresNetworkConnectivity = args.requiresNetwork ?? false
      processing.requiresExternalPower = args.requiresCharging ?? false
      request = processing
    } else {
      request = BGAppRefreshTaskRequest(identifier: args.name)
    }
    if let delay = delay, delay > 0 {
      request.earliestBeginDate = Date(timeIntervalSinceNow: delay / 1000)
    }
    // replaces the pending request with the same identifier
    try BGTaskScheduler.shared.submit(request)
  }

  private func send(_ event: TaskEvent) {
    lock.lock()
    guard let channel = channel else {
      pendingEvents.append(event)
      lock.unlock()
      return
    }
    lock.unlock()

    do {
      try channel.send(event)
    } catch {
      Logger.error("failed to send a background task event: \(error)")
    }
  }

  private func launch(_ task: BGTask) {
    let runId = UUID().uuidString
    let name = task.identifier
    let args = storedRequest(name)

    if let args = args {
      if let interval = args.interval {
        // the system runs a request once, so periodic tasks are submitted again on each run
        do {
          try submit(args, delay: interval)
        } catch {
          Logger.error("failed to schedule the next run of \(name): \(error)")
        }
      } else {
        // the handler may schedule the task again, which must survive this run
        removeStoredRequest(name)
      }
    }

    lock.lock()
    runs[runId] = Run(task: task, args: args)
    lock.unlock()

    task.expirationHandler = { [weak self] in
      self?.expire(runId)
    }

    send(TaskEvent(event: "run", runId: runId, name: name, input: args?.input ?? "null"))
  }

  /// The system wants the task to stop now. Tell it we are done and let the handler
  /// know; it can still ask for a retry when it returns.
  private func expire(_ runId: String) {
    lock.lock()
    let task = runs[runId]?.task
    runs[runId]?.task = nil
    lock.unlock()

    send(TaskEvent(event: "expire", runId: runId))
    task?.setTaskCompleted(success: false)
  }

  //
  // Tauri commands
  //

  @objc public func setHandler(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SetHandlerArgs.self)

    lock.lock()
    channel = args.channel
    let events = pendingEvents
    pendingEvents = []
    lock.unlock()

    for event in events {
      send(event)
    }

    invoke.resolve()
  }

  @objc public func complete(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CompleteArgs.self)

    lock.lock()
    let run = runs.removeValue(forKey: args.runId)
    lock.unlock()

    if let run = run {
      if args.outcome == "retry", let request = run.args {
        do {
          try store(request)
          try submit(request, delay: nil)
        } catch {
          Logger.error("failed to retry the background task \(request.name): \(error)")
        }
      }
      run.task?.setTaskCompleted(success: args.outcome == "success")
    }

    invoke.resolve()
  }

  @objc public func schedule(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(ScheduleArgs.self)

    do {
      try submit(args, delay: args.delay)
      try store(args)
      invoke.resolve()
    } catch let error as BGTaskScheduler.Error {
      switch error.code {
      case .notPermitted:
        invoke.reject(
          "\(args.name) is not listed in BGTaskSchedulerPermittedIdentifiers, or the app lacks the required background mode"
        )
      case .unavailable:
        invoke.reject(
          "background tasks are unavailable: Background App Refresh is disabled or the app runs on the simulator"
        )
      case .tooManyPendingTaskRequests:
        invoke.reject("too many pending background task requests")
      default:
        invoke.reject(error.localizedDescription)
      }
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }

  @objc public func cancel(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(CancelArgs.self)
    BGTaskScheduler.shared.cancel(taskRequestWithIdentifier: args.name)
    removeStoredRequest(args.name)
    invoke.resolve()
  }

  @objc public func cancelAll(_ invoke: Invoke) throws {
    // only the requests of this plugin, the app may schedule its own
    for name in storedRequests().keys {
      BGTaskScheduler.shared.cancel(taskRequestWithIdentifier: name)
      removeStoredRequest(name)
    }
    invoke.resolve()
  }

  @objc public func getScheduled(_ invoke: Invoke) throws {
    let names = Set(storedRequests().keys)
    BGTaskScheduler.shared.getPendingTaskRequests { requests in
      let tasks =
        requests
        .filter { names.contains($0.identifier) }
        .map { ScheduledTask(name: $0.identifier) }
      invoke.resolve(ScheduledResponse(tasks: tasks))
    }
  }
}

@_cdecl("init_plugin_background_tasks")
func initPlugin() -> Plugin {
  return BackgroundTasksPlugin()
}
