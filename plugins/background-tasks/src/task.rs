// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    any::Any,
    collections::HashMap,
    fmt::Display,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

use serde::{Serialize, de::DeserializeOwned};
use tauri::{AppHandle, Runtime};
use tokio::sync::watch;

type BoxFuture = Pin<Box<dyn Future<Output = Outcome> + Send>>;
type Handler = Arc<dyn Fn(TaskContext) -> BoxFuture + Send + Sync>;

/// Cancellation senders of the task runs in progress, keyed by run id.
static RUNS: Mutex<Option<HashMap<String, watch::Sender<bool>>>> = Mutex::new(None);

/// The `AppHandle<R>` of the running app, set when the plugin is initialized.
static APP: Mutex<Option<Box<dyn Any + Send + Sync>>> = Mutex::new(None);

pub(crate) fn set_app_handle<R: Runtime>(app: AppHandle<R>) {
    APP.lock().unwrap().replace(Box::new(app));
}

/// What the system should do with a task after its handler returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    /// The task finished its work.
    Success,
    /// The task could not finish now and should run again later.
    ///
    /// Android retries it with WorkManager's backoff policy. iOS submits the request
    /// again and runs it at the next opportunity.
    Retry,
    /// The task failed and should not be retried. A periodic task still runs again at
    /// its next interval.
    Failure,
}

impl From<()> for Outcome {
    fn from(_: ()) -> Self {
        Self::Success
    }
}

impl<T: Into<Outcome>, E: Display> From<Result<T, E>> for Outcome {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(outcome) => outcome.into(),
            Err(error) => {
                log::error!("background task failed: {error}");
                Self::Failure
            }
        }
    }
}

/// The task handlers of the app, by name.
///
/// Build it in a function so that both [`init`](crate::init) and
/// [`export_tasks!`](crate::export_tasks) can use it:
///
/// ```
/// use tauri_plugin_background_tasks::{Outcome, TaskContext, Tasks};
///
/// async fn sync(ctx: TaskContext) -> Outcome {
///     if ctx.is_cancelled() {
///         return Outcome::Retry;
///     }
///     Outcome::Success
/// }
///
/// fn tasks() -> Tasks {
///     Tasks::new().task("com.example.app.sync", sync)
/// }
///
/// tauri_plugin_background_tasks::export_tasks!(tasks);
/// ```
#[derive(Default, Clone)]
pub struct Tasks(HashMap<String, Handler>);

impl Tasks {
    /// Creates an empty set of tasks.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the handler of the task called `name`.
    ///
    /// The handler can return [`Outcome`], `()` (success) or a `Result` of those, whose
    /// error is logged and reported as [`Outcome::Failure`]. A handler that panics also
    /// fails the task.
    ///
    /// On iOS the name must be one of the `BGTaskSchedulerPermittedIdentifiers` in the
    /// app's `Info.plist`.
    pub fn task<F, Fut, O>(mut self, name: impl Into<String>, handler: F) -> Self
    where
        F: Fn(TaskContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = O> + Send + 'static,
        O: Into<Outcome>,
    {
        self.0.insert(
            name.into(),
            Arc::new(move |ctx| {
                let future = handler(ctx);
                Box::pin(async move { future.await.into() })
            }),
        );
        self
    }
}

impl std::fmt::Debug for Tasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.0.keys()).finish()
    }
}

/// What a task handler knows about the run it was started for.
#[derive(Debug, Clone)]
pub struct TaskContext {
    name: String,
    input: serde_json::Value,
    cancel: watch::Receiver<bool>,
}

impl TaskContext {
    /// The name the task was scheduled with.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The input the task was scheduled with, [`serde_json::Value::Null`] if none.
    pub fn input(&self) -> &serde_json::Value {
        &self.input
    }

    /// Deserializes the input the task was scheduled with.
    pub fn input_as<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_value(self.input.clone())
    }

    /// Whether the system asked the task to stop, because it ran out of time, its
    /// constraints are no longer met or it was cancelled.
    ///
    /// A stopped task should return as soon as it can. Return [`Outcome::Retry`] to run
    /// it again later.
    pub fn is_cancelled(&self) -> bool {
        *self.cancel.borrow()
    }

    /// Completes when the system asks the task to stop. See [`Self::is_cancelled`].
    pub async fn cancelled(&self) {
        let mut cancel = self.cancel.clone();
        if cancel.wait_for(|cancelled| *cancelled).await.is_err() {
            // the run is over, so it can't be cancelled anymore
            std::future::pending::<()>().await;
        }
    }

    /// The handle of the running app, if there is one.
    ///
    /// This is `None` when Android started the app's process only to run the task:
    /// the Tauri app is not created in that case, so the handler must not depend on it.
    /// It is also `None` if `R` is not the runtime the app uses.
    pub fn app_handle<R: Runtime>(&self) -> Option<AppHandle<R>> {
        APP.lock()
            .unwrap()
            .as_ref()
            .and_then(|app| app.downcast_ref::<AppHandle<R>>())
            .cloned()
    }
}

/// Runs the handler of the task called `name` to completion.
#[cfg_attr(desktop, allow(dead_code))]
pub(crate) async fn run(
    tasks: &Tasks,
    run_id: &str,
    name: &str,
    input: serde_json::Value,
) -> Outcome {
    let Some(handler) = tasks.0.get(name).cloned() else {
        log::error!("no background task handler is registered for `{name}`");
        return Outcome::Failure;
    };

    let (cancel_tx, cancel_rx) = watch::channel(false);
    RUNS.lock()
        .unwrap()
        .get_or_insert_default()
        .insert(run_id.to_string(), cancel_tx);

    let ctx = TaskContext {
        name: name.to_string(),
        input,
        cancel: cancel_rx,
    };
    // spawning the handler keeps a panic in it from taking the caller down
    let outcome = match tauri::async_runtime::spawn(async move { handler(ctx).await }).await {
        Ok(outcome) => outcome,
        Err(error) => {
            log::error!("background task `{name}` panicked: {error}");
            Outcome::Failure
        }
    };

    if let Some(runs) = RUNS.lock().unwrap().as_mut() {
        runs.remove(run_id);
    }

    outcome
}

/// Tells the handler of the given run that the system wants it to stop.
#[cfg_attr(desktop, allow(dead_code))]
pub(crate) fn cancel(run_id: &str) {
    if let Some(cancel) = RUNS.lock().unwrap().as_ref().and_then(|r| r.get(run_id)) {
        let _ = cancel.send(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_blocking(tasks: &Tasks, run_id: &str, name: &str) -> Outcome {
        tauri::async_runtime::block_on(run(tasks, run_id, name, serde_json::json!({ "n": 1 })))
    }

    #[test]
    fn runs_the_handler_with_its_input() {
        #[derive(serde::Deserialize)]
        struct Input {
            n: u8,
        }

        let tasks = Tasks::new().task("input", |ctx: TaskContext| async move {
            let input = ctx.input_as::<Input>()?;
            if ctx.name() == "input" && input.n == 1 {
                Ok::<_, serde_json::Error>(Outcome::Success)
            } else {
                Ok(Outcome::Failure)
            }
        });

        assert_eq!(run_blocking(&tasks, "input-run", "input"), Outcome::Success);
    }

    #[test]
    fn maps_return_values() {
        let tasks = Tasks::new()
            .task("unit", |_| async {})
            .task("retry", |_| async { Outcome::Retry })
            .task("error", |_| async { Err::<(), _>("failed") });

        assert_eq!(run_blocking(&tasks, "unit-run", "unit"), Outcome::Success);
        assert_eq!(run_blocking(&tasks, "retry-run", "retry"), Outcome::Retry);
        assert_eq!(run_blocking(&tasks, "error-run", "error"), Outcome::Failure);
    }

    #[test]
    fn unknown_task_fails() {
        assert_eq!(
            run_blocking(&Tasks::new(), "unknown-run", "unknown"),
            Outcome::Failure
        );
    }

    #[test]
    fn panic_fails_the_task() {
        let tasks = Tasks::new().task("panic", |ctx: TaskContext| async move {
            if ctx.name() == "panic" {
                panic!("task panicked");
            }
        });

        assert_eq!(run_blocking(&tasks, "panic-run", "panic"), Outcome::Failure);
    }

    #[test]
    fn cancel_reaches_the_handler() {
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let tasks = Tasks::new().task("cancel", move |ctx: TaskContext| {
            let started_tx = started_tx.clone();
            async move {
                started_tx.send(()).unwrap();
                ctx.cancelled().await;
                assert!(ctx.is_cancelled());
                Outcome::Retry
            }
        });

        let canceller = std::thread::spawn(move || {
            started_rx.recv().unwrap();
            // only the matching run is cancelled
            cancel("another-run");
            cancel("cancel-run");
        });

        assert_eq!(run_blocking(&tasks, "cancel-run", "cancel"), Outcome::Retry);
        canceller.join().unwrap();
    }
}
