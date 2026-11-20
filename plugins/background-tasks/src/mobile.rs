// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::{ScheduledTask, TaskKind, TaskRequest, Tasks};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.backgroundtasks";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_background_tasks);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
    tasks: Tasks,
) -> crate::Result<BackgroundTasks<R>> {
    #[cfg(target_os = "android")]
    let handle = {
        // Android reaches the handlers through the symbols `export_tasks!` exports
        let _ = tasks;
        api.register_android_plugin(PLUGIN_IDENTIFIER, "BackgroundTasksPlugin")?
    };
    #[cfg(target_os = "ios")]
    let handle = {
        let handle = api.register_ios_plugin(init_plugin_background_tasks)?;
        ios::set_handler(&handle, tasks)?;
        handle
    };
    Ok(BackgroundTasks(handle))
}

/// Access to the background tasks APIs.
pub struct BackgroundTasks<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> BackgroundTasks<R> {
    /// Schedules a task, replacing the pending request with the same name.
    ///
    /// The system decides when the task actually runs. On iOS this fails if the name is
    /// not one of the app's `BGTaskSchedulerPermittedIdentifiers`, if the user disabled
    /// Background App Refresh, and on the simulator.
    pub fn schedule(&self, request: TaskRequest) -> crate::Result<()> {
        let payload = SchedulePayload {
            // the native side stores the input as an opaque string
            input: serde_json::to_string(&request.input)?,
            name: request.name,
            delay: request.delay,
            interval: request.interval,
            requires_network: request.requires_network,
            requires_charging: request.requires_charging,
            kind: request.kind,
        };
        self.0
            .run_mobile_plugin("schedule", payload)
            .map_err(Into::into)
    }

    /// Cancels the pending request of the task called `name`. On Android this also
    /// stops the task if it is running.
    pub fn cancel(&self, name: &str) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("cancel", NamePayload { name })
            .map_err(Into::into)
    }

    /// Cancels every pending request scheduled through this plugin.
    pub fn cancel_all(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("cancelAll", ())
            .map_err(Into::into)
    }

    /// Returns the tasks that are waiting to run.
    pub fn scheduled(&self) -> crate::Result<Vec<ScheduledTask>> {
        self.0
            .run_mobile_plugin::<ScheduledResponse>("getScheduled", ())
            .map(|response| response.tasks)
            .map_err(Into::into)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SchedulePayload {
    name: String,
    input: String,
    delay: Option<u64>,
    interval: Option<u64>,
    requires_network: bool,
    requires_charging: bool,
    kind: TaskKind,
}

#[derive(Serialize)]
struct NamePayload<'a> {
    name: &'a str,
}

#[derive(Deserialize)]
struct ScheduledResponse {
    tasks: Vec<ScheduledTask>,
}

#[cfg(target_os = "ios")]
mod ios {
    use serde::{Deserialize, Serialize};
    use tauri::{
        Runtime,
        ipc::{Channel, InvokeResponseBody},
        plugin::PluginHandle,
    };

    use crate::{Outcome, Tasks, task};

    /// Sent by the Swift plugin when the system launches a task or asks it to stop.
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct TaskEvent {
        event: String,
        run_id: String,
        name: Option<String>,
        input: Option<String>,
    }

    #[derive(Serialize)]
    struct HandlerPayload {
        channel: Channel,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CompletePayload {
        run_id: String,
        outcome: Outcome,
    }

    /// Hands the Swift plugin a channel through which it asks us to run tasks.
    pub fn set_handler<R: Runtime>(handle: &PluginHandle<R>, tasks: Tasks) -> crate::Result<()> {
        let handle_ = handle.clone();
        let channel = Channel::new(move |event| {
            let InvokeResponseBody::Json(payload) = event else {
                return Ok(());
            };
            match serde_json::from_str::<TaskEvent>(&payload) {
                Ok(event) => on_event(&handle_, &tasks, event),
                Err(error) => log::error!("invalid background task event: {error}"),
            }
            Ok(())
        });

        handle
            .run_mobile_plugin::<()>("setHandler", HandlerPayload { channel })
            .map_err(Into::into)
    }

    fn on_event<R: Runtime>(handle: &PluginHandle<R>, tasks: &Tasks, event: TaskEvent) {
        match event.event.as_str() {
            "run" => {
                let handle = handle.clone();
                let tasks = tasks.clone();
                tauri::async_runtime::spawn(async move {
                    let name = event.name.unwrap_or_default();
                    let input = event
                        .input
                        .and_then(|input| serde_json::from_str(&input).ok())
                        .unwrap_or_default();
                    let outcome = task::run(&tasks, &event.run_id, &name, input).await;
                    let payload = CompletePayload {
                        run_id: event.run_id,
                        outcome,
                    };
                    if let Err(error) = handle
                        .run_mobile_plugin_async::<()>("complete", payload)
                        .await
                    {
                        log::error!("failed to complete background task `{name}`: {error}");
                    }
                });
            }
            "expire" => task::cancel(&event.run_id),
            other => log::error!("unknown background task event `{other}`"),
        }
    }
}
