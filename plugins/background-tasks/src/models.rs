// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// How much runtime a task needs. Only iOS distinguishes the two kinds.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskKind {
    /// A short task that refreshes the app's content.
    ///
    /// On iOS this is a `BGAppRefreshTask`: it gets around 30 seconds and requires the
    /// `fetch` background mode. [`TaskRequest::requires_network`] and
    /// [`TaskRequest::requires_charging`] are ignored for it.
    #[default]
    Refresh,
    /// A longer task such as a database cleanup or a large upload.
    ///
    /// On iOS this is a `BGProcessingTask`: it gets several minutes, usually while the
    /// device is idle, and requires the `processing` background mode.
    Processing,
}

/// A request to run one of the tasks registered with [`Tasks`](crate::Tasks).
///
/// Scheduling a task replaces the pending request with the same name, if any.
///
/// ```
/// use std::time::Duration;
/// use tauri_plugin_background_tasks::TaskRequest;
///
/// let request = TaskRequest::new("com.example.app.sync")
///     .input(serde_json::json!({ "full": true }))
///     .interval(Duration::from_secs(60 * 60))
///     .requires_network(true);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TaskRequest {
    /// The name the task handler was registered with.
    ///
    /// On iOS this is also the task identifier, which must be listed in the
    /// `BGTaskSchedulerPermittedIdentifiers` array of the app's `Info.plist`.
    pub name: String,
    /// JSON value handed to the task handler. Android limits it to about 10 KB.
    #[serde(default)]
    pub input: serde_json::Value,
    /// Minimum time to wait before the task first runs, in milliseconds.
    #[serde(default)]
    pub delay: Option<u64>,
    /// Run the task repeatedly, at most once per this many milliseconds.
    /// Android raises anything below 15 minutes to 15 minutes.
    #[serde(default)]
    pub interval: Option<u64>,
    /// Only run the task while the device has a network connection.
    #[serde(default)]
    pub requires_network: bool,
    /// Only run the task while the device is charging.
    #[serde(default)]
    pub requires_charging: bool,
    /// How much runtime the task needs.
    #[serde(default)]
    pub kind: TaskKind,
}

impl TaskRequest {
    /// Creates a request that runs the task registered as `name` once, as soon as the
    /// system allows it.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            input: serde_json::Value::Null,
            delay: None,
            interval: None,
            requires_network: false,
            requires_charging: false,
            kind: TaskKind::default(),
        }
    }

    /// Sets the JSON value handed to the task handler.
    pub fn input(mut self, input: impl Into<serde_json::Value>) -> Self {
        self.input = input.into();
        self
    }

    /// Sets the minimum time to wait before the task first runs.
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = Some(millis(delay));
        self
    }

    /// Makes the task periodic: it runs at most once per `interval` until cancelled.
    ///
    /// The system decides the actual times. Android enforces a minimum of 15 minutes and
    /// iOS runs tasks only as often as the user's usage of the app justifies.
    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = Some(millis(interval));
        self
    }

    /// Only run the task while the device has a network connection.
    pub fn requires_network(mut self, requires_network: bool) -> Self {
        self.requires_network = requires_network;
        self
    }

    /// Only run the task while the device is charging.
    pub fn requires_charging(mut self, requires_charging: bool) -> Self {
        self.requires_charging = requires_charging;
        self
    }

    /// Sets how much runtime the task needs.
    pub fn kind(mut self, kind: TaskKind) -> Self {
        self.kind = kind;
        self
    }
}

fn millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

/// A task that is waiting to run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ScheduledTask {
    /// The name the task was scheduled with.
    pub name: String,
}
