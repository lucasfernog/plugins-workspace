// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Simple, persistent key-value store.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

pub use error::{Error, Result};
use serde::{Deserialize, Serialize};
pub use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
pub use store::{resolve_store_path, DeserializeFn, SerializeFn, Store, StoreBuilder};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, ResourceId, RunEvent, Runtime, State,
};

mod error;
mod store;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChangePayload<'a> {
    path: &'a Path,
    resource_id: Option<u32>,
    key: &'a str,
    value: Option<&'a JsonValue>,
    exists: bool,
}

#[derive(Debug)]
struct StoreState {
    stores: Arc<RwLock<HashMap<PathBuf, ResourceId>>>,
    /// Held while a store is being created, see `StoreBuilder::build_inner`.
    build_lock: Mutex<()>,
    serialize_fns: HashMap<String, SerializeFn>,
    deserialize_fns: HashMap<String, DeserializeFn>,
    default_serialize: SerializeFn,
    default_deserialize: DeserializeFn,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AutoSave {
    DebounceDuration(u64),
    Bool(bool),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadStoreOptions {
    defaults: Option<HashMap<String, JsonValue>>,
    auto_save: Option<AutoSave>,
    serialize_fn_name: Option<String>,
    deserialize_fn_name: Option<String>,
    #[serde(default)]
    create_new: bool,
    #[serde(default)]
    override_defaults: bool,
}

fn builder<R: Runtime>(
    app: AppHandle<R>,
    store_state: State<'_, StoreState>,
    path: PathBuf,
    options: Option<LoadStoreOptions>,
) -> Result<StoreBuilder<R>> {
    let mut builder = app.store_builder(path);

    let Some(options) = options else {
        return Ok(builder);
    };

    if let Some(defaults) = options.defaults {
        builder = builder.defaults(defaults);
    }

    if let Some(auto_save) = options.auto_save {
        match auto_save {
            AutoSave::DebounceDuration(duration) => {
                builder = builder.auto_save(Duration::from_millis(duration));
            }
            AutoSave::Bool(false) => {
                builder = builder.disable_auto_save();
            }
            _ => {}
        }
    }

    if let Some(serialize_fn_name) = options.serialize_fn_name {
        let serialize_fn = store_state
            .serialize_fns
            .get(&serialize_fn_name)
            .ok_or_else(|| crate::Error::SerializeFunctionNotFound(serialize_fn_name))?;
        builder = builder.serialize(*serialize_fn);
    }

    if let Some(deserialize_fn_name) = options.deserialize_fn_name {
        let deserialize_fn = store_state
            .deserialize_fns
            .get(&deserialize_fn_name)
            .ok_or_else(|| crate::Error::DeserializeFunctionNotFound(deserialize_fn_name))?;
        builder = builder.deserialize(*deserialize_fn);
    }

    if options.create_new {
        builder = builder.create_new();
    }

    if options.override_defaults {
        builder = builder.override_defaults();
    }

    Ok(builder)
}

#[tauri::command]
async fn load<R: Runtime>(
    app: AppHandle<R>,
    store_state: State<'_, StoreState>,
    path: PathBuf,
    options: Option<LoadStoreOptions>,
) -> Result<ResourceId> {
    let builder = builder(app, store_state, path, options)?;
    let (_, rid) = builder.build_inner()?;
    Ok(rid)
}

#[tauri::command]
async fn get_store<R: Runtime>(
    app: AppHandle<R>,
    store_state: State<'_, StoreState>,
    path: PathBuf,
) -> Result<Option<ResourceId>> {
    let stores = store_state.stores.read().unwrap();
    Ok(stores.get(&resolve_store_path(&app, path)?).copied())
}

#[tauri::command]
async fn set<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    key: String,
    value: JsonValue,
) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.set(key, value);
    Ok(())
}

#[tauri::command]
async fn get<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    key: String,
) -> Result<(Option<JsonValue>, bool)> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    let value = store.get(key);
    let exists = value.is_some();
    Ok((value, exists))
}

#[tauri::command]
async fn has<R: Runtime>(app: AppHandle<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.has(key))
}

#[tauri::command]
async fn delete<R: Runtime>(app: AppHandle<R>, rid: ResourceId, key: String) -> Result<bool> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.delete(key))
}

#[tauri::command]
async fn clear<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.clear();
    Ok(())
}

#[tauri::command]
async fn reset<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.reset();
    Ok(())
}

#[tauri::command]
async fn keys<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<Vec<String>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.keys())
}

#[tauri::command]
async fn values<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<Vec<JsonValue>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.values())
}

#[tauri::command]
async fn entries<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
) -> Result<Vec<(String, JsonValue)>> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.entries())
}

#[tauri::command]
async fn length<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<usize> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    Ok(store.length())
}

#[tauri::command]
async fn reload<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
    ignore_defaults: Option<bool>,
) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    if ignore_defaults.unwrap_or_default() {
        store.reload_ignore_defaults()
    } else {
        store.reload()
    }
}

#[tauri::command]
async fn save<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> Result<()> {
    let store = app.resources_table().get::<Store<R>>(rid)?;
    store.save()
}

/// Extension trait to access the store APIs on a [`Manager`] such as `App`, `AppHandle`,
/// `WebviewWindow` or `Window`.
///
/// The plugin must be registered with [`Builder::build`] for these methods to work,
/// as they rely on the state it manages.
pub trait StoreExt<R: Runtime> {
    /// Create a store or load an existing store with default settings at the given path.
    ///
    /// If the store is already loaded, its instance is automatically returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::StoreExt;
    ///
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = app.store("my-store")?;
    ///     Ok(())
    ///   });
    /// ```
    fn store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>>;
    /// Get a store builder.
    ///
    /// The builder can be used to configure the store.
    /// To use the default settings see [`Self::store`].
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::StoreExt;
    /// use std::time::Duration;
    ///
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = app.store_builder("users.json").auto_save(Duration::from_secs(1)).build()?;
    ///     Ok(())
    ///   });
    /// ```
    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R>;
    /// Get a handle of an already loaded store.
    ///
    /// If the store is not loaded or does not exist, it returns `None`.
    ///
    /// Note that using this function can cause race conditions if you fallback to creating or loading the store,
    /// so you should consider using [`Self::store`] if you are not sure if the store is loaded or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_store::StoreExt;
    ///
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = if let Some(s) = app.get_store("store.json") {
    ///       s
    ///     } else {
    ///       // this is not thread safe; if another thread is doing the same load/create,
    ///       // there will be a race condition; in this case we could remove the get_store
    ///       // and only run app.store() as it will return the existing store if it has been loaded
    ///       app.store("store.json")?
    ///     };
    ///     Ok(())
    ///   });
    /// ```
    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>>;
}

impl<R: Runtime, T: Manager<R>> StoreExt<R> for T {
    fn store(&self, path: impl AsRef<Path>) -> Result<Arc<Store<R>>> {
        StoreBuilder::new(self.app_handle(), path).build()
    }

    fn store_builder(&self, path: impl AsRef<Path>) -> StoreBuilder<R> {
        StoreBuilder::new(self.app_handle(), path)
    }

    fn get_store(&self, path: impl AsRef<Path>) -> Option<Arc<Store<R>>> {
        let path = resolve_store_path(self.app_handle(), path.as_ref()).ok()?;
        // release the `stores` lock before locking the resources table (see `build_inner`)
        let rid = self
            .state::<StoreState>()
            .stores
            .read()
            .unwrap()
            .get(&path)
            .copied()?;
        self.resources_table().get(rid).ok()
    }
}

fn default_serialize(
    cache: &HashMap<String, JsonValue>,
) -> std::result::Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::to_vec_pretty(&cache)?)
}

fn default_deserialize(
    bytes: &[u8],
) -> std::result::Result<HashMap<String, JsonValue>, Box<dyn std::error::Error + Send + Sync>> {
    serde_json::from_slice(bytes).map_err(Into::into)
}

/// Builder for the store plugin.
///
/// It is used to register custom serialize and deserialize functions the frontend can select by
/// name when loading a store, and to change the functions used by default (pretty printed JSON).
///
/// # Examples
///
/// ```
/// tauri::Builder::default()
///   .plugin(tauri_plugin_store::Builder::default().build());
/// ```
pub struct Builder {
    serialize_fns: HashMap<String, SerializeFn>,
    deserialize_fns: HashMap<String, DeserializeFn>,
    default_serialize: SerializeFn,
    default_deserialize: DeserializeFn,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            serialize_fns: Default::default(),
            deserialize_fns: Default::default(),
            default_serialize,
            default_deserialize,
        }
    }
}

impl Builder {
    /// Creates a new builder using the default serialize and deserialize functions,
    /// which read and write pretty printed JSON.
    ///
    /// This is the same as [`Builder::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a serialize function to access it from the JavaScript side
    ///
    /// # Examples
    ///
    /// ```
    /// fn no_pretty_json(
    ///     cache: &std::collections::HashMap<String, serde_json::Value>,
    /// ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    ///     Ok(serde_json::to_vec(&cache)?)
    /// }
    ///
    /// tauri::Builder::default()
    ///     .plugin(
    ///         tauri_plugin_store::Builder::default()
    ///             .register_serialize_fn("no-pretty-json".to_owned(), no_pretty_json)
    ///             .build(),
    ///     );
    /// ```
    pub fn register_serialize_fn(mut self, name: String, serialize_fn: SerializeFn) -> Self {
        self.serialize_fns.insert(name, serialize_fn);
        self
    }

    /// Register a deserialize function to access it from the JavaScript side
    pub fn register_deserialize_fn(mut self, name: String, deserialize_fn: DeserializeFn) -> Self {
        self.deserialize_fns.insert(name, deserialize_fn);
        self
    }

    /// Use this serialize function for stores by default
    ///
    /// # Examples
    ///
    /// ```
    /// fn no_pretty_json(
    ///     cache: &std::collections::HashMap<String, serde_json::Value>,
    /// ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    ///     Ok(serde_json::to_vec(&cache)?)
    /// }
    ///
    /// tauri::Builder::default()
    ///     .plugin(
    ///         tauri_plugin_store::Builder::default()
    ///             .default_serialize_fn(no_pretty_json)
    ///             .build(),
    ///     );
    /// ```
    pub fn default_serialize_fn(mut self, serialize_fn: SerializeFn) -> Self {
        self.default_serialize = serialize_fn;
        self
    }

    /// Use this deserialize function for stores by default
    pub fn default_deserialize_fn(mut self, deserialize_fn: DeserializeFn) -> Self {
        self.default_deserialize = deserialize_fn;
        self
    }

    /// Builds the plugin.
    ///
    /// # Examples
    ///
    /// ```
    /// tauri::Builder::default()
    ///   .plugin(tauri_plugin_store::Builder::default().build())
    ///   .setup(|app| {
    ///     let store = tauri_plugin_store::StoreBuilder::new(app, "store.bin").build()?;
    ///     Ok(())
    ///   });
    /// ```
    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        plugin::Builder::new("store")
            .invoke_handler(tauri::generate_handler![
                load, get_store, set, get, has, delete, clear, reset, keys, values, length,
                entries, reload, save,
            ])
            .setup(move |app_handle, _api| {
                app_handle.manage(StoreState {
                    stores: Arc::new(RwLock::new(HashMap::new())),
                    build_lock: Mutex::new(()),
                    serialize_fns: self.serialize_fns,
                    deserialize_fns: self.deserialize_fns,
                    default_serialize: self.default_serialize,
                    default_deserialize: self.default_deserialize,
                });
                Ok(())
            })
            .on_event(|app_handle, event| {
                if let RunEvent::Exit = event {
                    // release the `stores` lock before locking the resources table
                    // (see `build_inner`)
                    let stores: Vec<(PathBuf, ResourceId)> = app_handle
                        .state::<StoreState>()
                        .stores
                        .read()
                        .unwrap()
                        .iter()
                        .map(|(path, rid)| (path.clone(), *rid))
                        .collect();
                    for (path, rid) in stores {
                        let store = app_handle.resources_table().get::<Store<R>>(rid);
                        if let Ok(store) = store {
                            if let Err(err) = store.save() {
                                tracing::error!("failed to save store {path:?} with error {err:?}");
                            }
                        }
                    }
                }
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::channel;
    use tauri::{test::MockRuntime, Listener};

    fn mock_app() -> tauri::App<MockRuntime> {
        tauri::test::mock_builder()
            .plugin(Builder::new().build())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap()
    }

    /// An absolute path in a fresh temporary directory, so the tests never touch the real
    /// app data directory.
    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tauri-plugin-store-lib-test-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Runs `f` on another thread and fails if it does not finish in time (i.e. it deadlocked).
    fn run_with_timeout(f: impl FnOnce() + Send + 'static) {
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            f();
            let _ = tx.send(());
        });
        rx.recv_timeout(Duration::from_secs(10))
            .expect("the operation deadlocked");
    }

    #[test]
    fn change_listener_can_access_stores() {
        let app = mock_app();
        let dir = temp_dir("change-listener");
        let path = dir.join("listened.json");
        let other_path = dir.join("other.json");

        let store = app
            .store_builder(&path)
            .disable_auto_save()
            .build()
            .unwrap();

        let (events_tx, events_rx) = channel();
        let handle = app.handle().clone();
        let listened_path = path.clone();
        app.listen("store://change", move |_event| {
            // re-locks the store that emitted the event
            let store = handle.get_store(&listened_path).unwrap();
            let value = store.get("key");
            // takes the write lock of the stores registry
            let other = handle
                .store_builder(&other_path)
                .disable_auto_save()
                .build()
                .unwrap();
            let _ = other.keys();
            let _ = events_tx.send(value);
        });

        let store_ = store.clone();
        run_with_timeout(move || {
            store_.set("key", 1);
            store_.reset();
            store_.set("key", 2);
            store_.delete("key");
        });

        let events: Vec<_> = events_rx.try_iter().collect();
        assert_eq!(
            events,
            vec![
                Some(JsonValue::from(1)),
                None,
                Some(JsonValue::from(2)),
                None
            ]
        );

        drop(store);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn stale_store_handles_do_not_affect_their_replacement() {
        let app = mock_app();
        let dir = temp_dir("stale-handles");
        let path = dir.join("store.json");

        // replaced with `create_new`
        let (old, old_rid) = app
            .store_builder(&path)
            .disable_auto_save()
            .build_inner()
            .unwrap();
        let (new, new_rid) = app
            .store_builder(&path)
            .disable_auto_save()
            .create_new()
            .build_inner()
            .unwrap();
        assert_ne!(old_rid, new_rid);

        let (events_tx, events_rx) = channel();
        app.listen("store://change", move |event| {
            let payload: JsonValue = serde_json::from_str(event.payload()).unwrap();
            let _ = events_tx.send(payload["resourceId"].as_u64());
        });
        old.set("key", "stale");
        new.set("key", "fresh");
        assert_eq!(
            events_rx.try_iter().collect::<Vec<_>>(),
            vec![Some(old_rid as u64), Some(new_rid as u64)]
        );

        old.close_resource();
        let current = app.get_store(&path).expect("the new store was closed");
        assert!(Arc::ptr_eq(&current, &new));

        // closed, then loaded again
        new.close_resource();
        assert!(app.get_store(&path).is_none());
        let reloaded = app
            .store_builder(&path)
            .disable_auto_save()
            .build()
            .unwrap();
        new.close_resource();
        let current = app.get_store(&path).expect("the reloaded store was closed");
        assert!(Arc::ptr_eq(&current, &reloaded));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn concurrent_close_and_load_do_not_deadlock() {
        let app = mock_app();
        let dir = temp_dir("close-and-load");

        let mut threads = Vec::new();
        for thread in 0..4 {
            let handle = app.handle().clone();
            let path = dir.join(format!("store-{thread}.json"));
            threads.push(move || {
                for _ in 0..200 {
                    let store = handle
                        .store_builder(&path)
                        .disable_auto_save()
                        .build()
                        .unwrap();
                    let _ = handle.get_store(&path);
                    // closes through the resources table, like `close()` from JS
                    store.close_resource();
                }
            });
        }

        run_with_timeout(move || {
            let threads: Vec<_> = threads.into_iter().map(std::thread::spawn).collect();
            for thread in threads {
                thread.join().unwrap();
            }
        });

        let _ = std::fs::remove_dir_all(dir);
    }
}
