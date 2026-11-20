// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Rust side of the JNI functions [`export_tasks!`](crate::export_tasks) exports.
//! `TaskRunner.kt` declares them and `BackgroundTaskWorker.kt` calls them.

use std::{ffi::c_void, panic::AssertUnwindSafe};

use jni::{JNIEnv, objects::JString};

use crate::{Outcome, Tasks, task};

// result codes, mirrored in BackgroundTaskWorker.kt
const SUCCESS: i32 = 0;
const RETRY: i32 = 1;
const FAILURE: i32 = 2;

unsafe fn string(env: &mut JNIEnv<'_>, raw: *mut c_void) -> Option<String> {
    let string = unsafe { JString::from_raw(raw.cast()) };
    env.get_string(&string).ok().map(Into::into)
}

/// Runs the task called `name` on the calling thread until it completes.
///
/// # Safety
///
/// `env` must be the `JNIEnv` of the calling thread and the other pointers valid
/// `jstring` references, as the JVM passes them to a native method.
pub unsafe fn run(
    env: *mut c_void,
    run_id: *mut c_void,
    name: *mut c_void,
    input: *mut c_void,
    tasks: fn() -> Tasks,
) -> i32 {
    // a panic must not unwind into the JVM
    let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let mut env = unsafe { JNIEnv::from_raw(env.cast()) }.ok()?;
        let run_id = unsafe { string(&mut env, run_id) }?;
        let name = unsafe { string(&mut env, name) }?;
        let input = unsafe { string(&mut env, input) }
            .and_then(|input| serde_json::from_str(&input).ok())
            .unwrap_or_default();

        Some(tauri::async_runtime::block_on(task::run(
            &tasks(),
            &run_id,
            &name,
            input,
        )))
    }));

    match outcome {
        Ok(Some(Outcome::Success)) => SUCCESS,
        Ok(Some(Outcome::Retry)) => RETRY,
        Ok(Some(Outcome::Failure)) => FAILURE,
        Ok(None) => {
            log::error!("invalid arguments for a background task run");
            FAILURE
        }
        Err(_) => {
            log::error!("panicked while running a background task");
            FAILURE
        }
    }
}

/// Tells the handler of the given run that WorkManager stopped it.
///
/// # Safety
///
/// `env` must be the `JNIEnv` of the calling thread and `run_id` a valid `jstring`
/// reference, as the JVM passes them to a native method.
pub unsafe fn cancel(env: *mut c_void, run_id: *mut c_void) {
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        if let Ok(mut env) = unsafe { JNIEnv::from_raw(env.cast()) }
            && let Some(run_id) = unsafe { string(&mut env, run_id) }
        {
            task::cancel(&run_id);
        }
    }));
}
