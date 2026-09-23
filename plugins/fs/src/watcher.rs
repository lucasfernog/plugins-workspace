// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent, Debouncer, RecommendedCache};
use serde::Deserialize;
use tauri::{
    ipc::{Channel, CommandScope, GlobalScope},
    path::BaseDirectory,
    Manager, Resource, ResourceId, Runtime, Webview,
};

use std::time::Duration;

use crate::{
    commands::{resolve_path_checked, CommandResult, ForbiddenPatterns, ResolvedPath},
    scope::Entry,
    SafeFilePath,
};

#[allow(unused)]
enum WatcherKind {
    Debouncer(Debouncer<RecommendedWatcher, RecommendedCache>),
    Watcher(RecommendedWatcher),
}

impl Resource for WatcherKind {}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchOptions {
    base_dir: Option<BaseDirectory>,
    #[serde(default)]
    recursive: bool,
    delay_ms: Option<u64>,
}

#[tauri::command]
pub fn watch<R: Runtime>(
    webview: Webview<R>,
    paths: Vec<SafeFilePath>,
    options: WatchOptions,
    on_event: Channel<notify::Event>,
    global_scope: GlobalScope<Entry>,
    command_scope: CommandScope<Entry>,
) -> CommandResult<ResourceId> {
    let resolved_paths = paths
        .into_iter()
        .map(|path| {
            resolve_path_checked(
                "watch",
                &webview,
                &global_scope,
                &command_scope,
                path,
                options.base_dir,
            )
        })
        .collect::<CommandResult<Vec<_>>>()?;

    // the scope only checked the watched paths themselves,
    // do not report changes of the entries denied by the scope
    let forbidden = resolved_paths
        .first()
        .map(|resolved| resolved.forbidden.clone());
    let filter = move |event: notify::Event| {
        if let Some(forbidden) = &forbidden {
            filter_forbidden_paths(event, forbidden)
        } else {
            Some(event)
        }
    };
    let resolved_paths = resolved_paths
        .into_iter()
        .map(|ResolvedPath { handle, .. }| handle)
        .collect::<Vec<_>>();

    let recursive_mode = if options.recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    let watcher_kind = if let Some(delay) = options.delay_ms {
        let mut debouncer = new_debouncer(
            Duration::from_millis(delay),
            None,
            move |events: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
                if let Ok(events) = events {
                    for event in events {
                        // TODO: Should errors be emitted too?
                        if let Some(event) = filter(event.event) {
                            let _ = on_event.send(event);
                        }
                    }
                }
            },
        )?;
        for path in &resolved_paths {
            debouncer.watch(path, recursive_mode)?;
        }
        WatcherKind::Debouncer(debouncer)
    } else {
        let mut watcher = RecommendedWatcher::new(
            move |event| {
                if let Ok(event) = event {
                    // TODO: Should errors be emitted too?
                    if let Some(event) = filter(event) {
                        let _ = on_event.send(event);
                    }
                }
            },
            Config::default(),
        )?;
        for path in &resolved_paths {
            watcher.watch(path, recursive_mode)?;
        }
        WatcherKind::Watcher(watcher)
    };

    let rid = webview.resources_table().add(watcher_kind);

    Ok(rid)
}

/// Removes the paths denied by the scope from `event`, dropping it if none is left.
fn filter_forbidden_paths(
    mut event: notify::Event,
    forbidden: &ForbiddenPatterns,
) -> Option<notify::Event> {
    if event.paths.is_empty() {
        return Some(event);
    }
    event.paths.retain(|path| !forbidden.matches(path));
    (!event.paths.is_empty()).then_some(event)
}
