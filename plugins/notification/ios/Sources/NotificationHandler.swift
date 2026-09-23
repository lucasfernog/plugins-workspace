// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Tauri
import UserNotifications

public class NotificationHandler: NSObject, NotificationHandlerProtocol {

  public weak var plugin: Plugin?

  private var notificationsMap = [String: Notification]()

  internal func saveNotification(_ key: String, _ notification: Notification) {
    notificationsMap.updateValue(notification, forKey: key)
  }

  public func requestPermissions(with completion: ((Bool, Error?) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.requestAuthorization(options: [.badge, .alert, .sound]) { (granted, error) in
      completion?(granted, error)
    }
  }

  public func checkPermissions(with completion: ((UNAuthorizationStatus) -> Void)? = nil) {
    let center = UNUserNotificationCenter.current()
    center.getNotificationSettings { settings in
      completion?(settings.authorizationStatus)
    }
  }

  public func willPresent(notification: UNNotification) -> UNNotificationPresentationOptions {
    let notificationData = toActiveNotification(notification.request)
    try? self.plugin?.trigger("notification", data: notificationData)

    if let options = notificationsMap[notification.request.identifier] {
      if options.silent ?? false {
        return UNNotificationPresentationOptions.init(rawValue: 0)
      }
    }

    return [
      .badge,
      .sound,
      .alert,
    ]
  }

  public func didReceive(response: UNNotificationResponse) {
    let originalNotificationRequest = response.notification.request
    let actionId = response.actionIdentifier

    var actionIdValue: String
    // We turn the two default actions (open/dismiss) into generic strings
    if actionId == UNNotificationDefaultActionIdentifier {
      actionIdValue = "tap"
    } else if actionId == UNNotificationDismissActionIdentifier {
      actionIdValue = "dismiss"
    } else {
      actionIdValue = actionId
    }

    var inputValue: String? = nil
    // If the type of action was for an input type, get the value
    if let inputType = response as? UNTextInputNotificationResponse {
      inputValue = inputType.userText
    }

    try? self.plugin?.trigger(
      "actionPerformed",
      data: ReceivedNotification(
        actionId: actionIdValue,
        inputValue: inputValue,
        notification: toActiveNotification(originalNotificationRequest)
      ))
  }

  func toActiveNotification(_ request: UNNotificationRequest) -> ActiveNotification {
    let notificationRequest = notificationsMap[request.identifier]!
    return ActiveNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      sound: notificationRequest.sound ?? "",
      actionTypeId: request.content.categoryIdentifier,
      attachments: notificationRequest.attachments
    )
  }

  /// Returns `nil` for requests without a time based trigger, which are not scheduled notifications.
  func toPendingNotification(_ request: UNNotificationRequest) -> PendingNotification? {
    guard let schedule = PendingSchedule(request.trigger) else {
      return nil
    }
    return PendingNotification(
      id: Int(request.identifier) ?? -1,
      title: request.content.title,
      body: request.content.body,
      schedule: schedule
    )
  }
}

struct PendingNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let schedule: PendingSchedule
}

/// The schedule of a pending notification, in the shape of the JS `Schedule` and Rust `Schedule` types.
struct PendingSchedule: Encodable {
  struct At: Encodable {
    let date: String
    let repeating: Bool
    let allowWhileIdle = false
  }

  struct Interval: Encodable {
    let interval: ScheduleInterval
    let allowWhileIdle = false
  }

  var at: At?
  var interval: Interval?

  init?(_ trigger: UNNotificationTrigger?) {
    if let trigger = trigger as? UNCalendarNotificationTrigger {
      let components = trigger.dateComponents
      interval = Interval(
        interval: ScheduleInterval(
          year: components.year,
          month: components.month,
          day: components.day,
          weekday: components.weekday,
          hour: components.hour,
          minute: components.minute,
          second: components.second
        ))
    } else if let trigger = trigger as? UNTimeIntervalNotificationTrigger,
      let date = trigger.nextTriggerDate()
    {
      // `at` and `every` schedules are both time interval triggers: report the next delivery
      let formatter = DateFormatter()
      formatter.locale = Locale(identifier: "en_US_POSIX")
      formatter.timeZone = TimeZone(identifier: "UTC")
      formatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'"
      at = At(date: formatter.string(from: date), repeating: trigger.repeats)
    } else {
      return nil
    }
  }
}

struct ActiveNotification: Encodable {
  let id: Int
  let title: String
  let body: String
  let sound: String
  let actionTypeId: String
  let attachments: [NotificationAttachment]?
}

struct ReceivedNotification: Encodable {
  let actionId: String
  let inputValue: String?
  let notification: ActiveNotification
}
