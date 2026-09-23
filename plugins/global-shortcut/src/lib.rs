// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Register global shortcuts.
//!
//! - Supported platforms: Windows, Linux and macOS.
//! - On macOS, registering and unregistering shortcuts must happen on the main thread, so every
//!   [`GlobalShortcut`] operation dispatches to it and blocks until it completes.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use global_hotkey::GlobalHotKeyEvent;
pub use global_hotkey::{
    hotkey::{Code, HotKey as Shortcut, Modifiers},
    GlobalHotKeyEvent as ShortcutEvent, HotKeyState as ShortcutState,
};
use serde::Serialize;
use tauri::{
    ipc::Channel,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

mod error;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

type HotKeyId = u32;
type HandlerFn<R> = Box<dyn Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static>;

/// Internal wrapper around a [`Shortcut`] that lets the shortcut-accepting APIs on
/// [`GlobalShortcut`] and [`Builder`] be generic over an already-parsed [`Shortcut`] or a
/// string accelerator such as `"CmdOrControl+Q"`.
pub struct ShortcutWrapper(Shortcut);

impl From<Shortcut> for ShortcutWrapper {
    fn from(value: Shortcut) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for ShortcutWrapper {
    type Error = global_hotkey::hotkey::HotKeyParseError;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Shortcut::from_str(value).map(ShortcutWrapper)
    }
}

struct RegisteredShortcut<R: Runtime> {
    shortcut: Shortcut,
    handler: Option<Arc<HandlerFn<R>>>,
}

struct GlobalHotKeyManager(global_hotkey::GlobalHotKeyManager);

/// SAFETY: we ensure it is run on main thread only
unsafe impl Send for GlobalHotKeyManager {}
/// SAFETY: we ensure it is run on main thread only
unsafe impl Sync for GlobalHotKeyManager {}

/// The global shortcut APIs, accessible through [`GlobalShortcutExt::global_shortcut`].
///
/// Registering, unregistering and checking a shortcut all dispatch the underlying
/// `global_hotkey` call to the app's main thread and block until it finishes, since macOS
/// requires the hotkey manager to run on the main thread.
pub struct GlobalShortcut<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    manager: Arc<GlobalHotKeyManager>,
    shortcuts: Arc<Mutex<HashMap<HotKeyId, RegisteredShortcut<R>>>>,
}

macro_rules! run_main_thread {
    ($handle:expr, $manager:expr, |$m:ident| $ex:expr) => {{
        let (tx, rx) = std::sync::mpsc::channel();
        let manager = $manager.clone();
        let task = move || {
            let f = |$m: &GlobalHotKeyManager| $ex;
            let _ = tx.send(f(&*manager));
        };
        $handle.run_on_main_thread(task)?;
        rx.recv()?
    }};
}

impl<R: Runtime> GlobalShortcut<R> {
    fn register_internal<F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static>(
        &self,
        shortcut: Shortcut,
        handler: Option<F>,
    ) -> Result<()> {
        let id = shortcut.id();
        let handler = handler.map(|h| Arc::new(Box::new(h) as HandlerFn<R>));
        run_main_thread!(self.app, self.manager, |m| m.0.register(shortcut))?;
        self.shortcuts
            .lock()
            .unwrap()
            .insert(id, RegisteredShortcut { shortcut, handler });
        Ok(())
    }

    fn register_multiple_internal<S, F>(&self, shortcuts: S, handler: Option<F>) -> Result<()>
    where
        S: IntoIterator<Item = Shortcut>,
        F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
    {
        let handler = handler.map(|h| Arc::new(Box::new(h) as HandlerFn<R>));

        let hotkeys = shortcuts.into_iter().collect::<Vec<_>>();

        // Register every shortcut in a single main thread dispatch, without holding the
        // shortcuts lock: the main thread may need that lock to dispatch a hotkey event, so
        // holding it while waiting on the main thread could deadlock.
        {
            let hotkeys = hotkeys.clone();
            run_main_thread!(self.app, self.manager, |m| register_all_or_none(
                &hotkeys,
                |h| m.0.register(h),
                |h| m.0.unregister(h)
            ))?;
        }

        let mut shortcuts = self.shortcuts.lock().unwrap();
        for shortcut in hotkeys {
            shortcuts.insert(
                shortcut.id(),
                RegisteredShortcut {
                    shortcut,
                    handler: handler.clone(),
                },
            );
        }

        Ok(())
    }
}

impl<R: Runtime> GlobalShortcut<R> {
    /// Register a shortcut. Returns an error if the shortcut is invalid or already registered
    /// by this [`GlobalShortcut`] instance.
    pub fn register<S>(&self, shortcut: S) -> Result<()>
    where
        S: TryInto<ShortcutWrapper>,
        S::Error: std::error::Error,
    {
        self.register_internal(
            try_into_shortcut(shortcut)?,
            None::<fn(&AppHandle<R>, &Shortcut, ShortcutEvent)>,
        )
    }

    /// Register a shortcut with a handler that is called with the app handle, the triggered
    /// [`Shortcut`] and the [`ShortcutEvent`] (whose `state` is [`ShortcutState::Pressed`] or
    /// [`ShortcutState::Released`]) whenever the shortcut's key combination is pressed or
    /// released.
    pub fn on_shortcut<S, F>(&self, shortcut: S, handler: F) -> Result<()>
    where
        S: TryInto<ShortcutWrapper>,
        S::Error: std::error::Error,
        F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
    {
        self.register_internal(try_into_shortcut(shortcut)?, Some(handler))
    }

    /// Register multiple shortcuts.
    ///
    /// If any of the shortcuts fails to register, the ones registered by this call are
    /// unregistered again and the error is returned, so either all or none are registered.
    pub fn register_multiple<S, T>(&self, shortcuts: S) -> Result<()>
    where
        S: IntoIterator<Item = T>,
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
    {
        let mut s = Vec::new();
        for shortcut in shortcuts {
            s.push(try_into_shortcut(shortcut)?);
        }
        self.register_multiple_internal(s, None::<fn(&AppHandle<R>, &Shortcut, ShortcutEvent)>)
    }

    /// Register multiple shortcuts with a handler.
    ///
    /// If any of the shortcuts fails to register, the ones registered by this call are
    /// unregistered again and the error is returned, so either all or none are registered.
    pub fn on_shortcuts<S, T, F>(&self, shortcuts: S, handler: F) -> Result<()>
    where
        S: IntoIterator<Item = T>,
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
        F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static,
    {
        let mut s = Vec::new();
        for shortcut in shortcuts {
            s.push(try_into_shortcut(shortcut)?);
        }
        self.register_multiple_internal(s, Some(handler))
    }

    /// Unregister a shortcut
    pub fn unregister<S: TryInto<ShortcutWrapper>>(&self, shortcut: S) -> Result<()>
    where
        S::Error: std::error::Error,
    {
        let shortcut = try_into_shortcut(shortcut)?;
        run_main_thread!(self.app, self.manager, |m| m.0.unregister(shortcut))?;
        self.shortcuts.lock().unwrap().remove(&shortcut.id());
        Ok(())
    }

    /// Unregister multiple shortcuts.
    pub fn unregister_multiple<T: TryInto<ShortcutWrapper>, S: IntoIterator<Item = T>>(
        &self,
        shortcuts: S,
    ) -> Result<()>
    where
        T::Error: std::error::Error,
    {
        let mut mapped_shortcuts = Vec::new();
        for shortcut in shortcuts {
            mapped_shortcuts.push(try_into_shortcut(shortcut)?);
        }

        {
            let mapped_shortcuts = mapped_shortcuts.clone();
            #[rustfmt::skip]
            run_main_thread!(self.app, self.manager, |m| m.0.unregister_all(&mapped_shortcuts))?;
        }

        let mut shortcuts = self.shortcuts.lock().unwrap();
        for s in mapped_shortcuts {
            shortcuts.remove(&s.id());
        }

        Ok(())
    }

    /// Unregister all registered shortcuts.
    pub fn unregister_all(&self) -> Result<()> {
        let mut shortcuts = self.shortcuts.lock().unwrap();
        let hotkeys = std::mem::take(&mut *shortcuts);
        let hotkeys = hotkeys.values().map(|s| s.shortcut).collect::<Vec<_>>();
        #[rustfmt::skip]
        let res = run_main_thread!(self.app, self.manager, |m| m.0.unregister_all(hotkeys.as_slice()));
        res.map_err(Into::into)
    }

    /// Determines whether the given shortcut is registered by this application or not.
    ///
    /// If the shortcut is registered by another application, it will still return `false`.
    pub fn is_registered<S: TryInto<ShortcutWrapper>>(&self, shortcut: S) -> bool
    where
        S::Error: std::error::Error,
    {
        if let Ok(shortcut) = try_into_shortcut(shortcut) {
            self.shortcuts.lock().unwrap().contains_key(&shortcut.id())
        } else {
            false
        }
    }
}

/// Extension trait for [`Manager`] implementors (e.g. [`AppHandle`]) that exposes access to the
/// global shortcut APIs.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::Runtime;
/// use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
///
/// fn setup<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
///     app.global_shortcut()
///         .on_shortcut("CommandOrControl+Shift+C", |_app, shortcut, event| {
///             if event.state == ShortcutState::Pressed {
///                 println!("{shortcut:?} pressed");
///             }
///         })?;
///
///     Ok(())
/// }
/// ```
pub trait GlobalShortcutExt<R: Runtime> {
    /// Returns the [`GlobalShortcut`] instance managed by this plugin.
    fn global_shortcut(&self) -> &GlobalShortcut<R>;
}

impl<R: Runtime, T: Manager<R>> GlobalShortcutExt<R> for T {
    fn global_shortcut(&self) -> &GlobalShortcut<R> {
        self.state::<GlobalShortcut<R>>().inner()
    }
}

/// Registers every hotkey with `register`. If one of them fails, the hotkeys registered so far
/// are unregistered again with `unregister` and the error is returned, so either all of them are
/// registered or none are.
fn register_all_or_none<E>(
    hotkeys: &[Shortcut],
    mut register: impl FnMut(Shortcut) -> std::result::Result<(), E>,
    mut unregister: impl FnMut(Shortcut) -> std::result::Result<(), E>,
) -> std::result::Result<(), E> {
    for (i, hotkey) in hotkeys.iter().enumerate() {
        if let Err(e) = register(*hotkey) {
            for registered in &hotkeys[..i] {
                let _ = unregister(*registered);
            }
            return Err(e);
        }
    }
    Ok(())
}

fn parse_shortcut<S: AsRef<str>>(shortcut: S) -> Result<Shortcut> {
    shortcut.as_ref().parse().map_err(Into::into)
}

fn try_into_shortcut<S: TryInto<ShortcutWrapper>>(shortcut: S) -> Result<Shortcut>
where
    S::Error: std::error::Error,
{
    shortcut
        .try_into()
        .map(|s| s.0)
        .map_err(|e| Error::GlobalHotkey(e.to_string()))
}

#[derive(Clone, Serialize)]
struct ShortcutJsEvent {
    shortcut: String,
    id: u32,
    state: ShortcutState,
}

#[tauri::command]
fn register<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcuts: Vec<String>,
    handler: Channel<ShortcutJsEvent>,
) -> Result<()> {
    let mut hotkeys = Vec::new();

    let mut shortcut_map = HashMap::new();
    for shortcut in shortcuts {
        let hotkey = parse_shortcut(&shortcut)?;
        shortcut_map.insert(hotkey.id(), shortcut);
        hotkeys.push(hotkey);
    }

    global_shortcut.register_multiple_internal(
        hotkeys,
        Some(
            move |_app: &AppHandle<R>, shortcut: &Shortcut, e: ShortcutEvent| {
                let js_event = ShortcutJsEvent {
                    id: e.id,
                    state: e.state,
                    shortcut: shortcut.into_string(),
                };
                let _ = handler.send(js_event);
            },
        ),
    )
}

#[tauri::command]
fn unregister<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcuts: Vec<String>,
) -> Result<()> {
    let mut hotkeys = Vec::new();
    for shortcut in shortcuts {
        hotkeys.push(parse_shortcut(&shortcut)?);
    }
    global_shortcut.unregister_multiple(hotkeys)
}

#[tauri::command]
fn unregister_all<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
) -> Result<()> {
    global_shortcut.unregister_all()
}

#[tauri::command]
fn is_registered<R: Runtime>(
    _app: AppHandle<R>,
    global_shortcut: State<'_, GlobalShortcut<R>>,
    shortcut: String,
) -> Result<bool> {
    Ok(global_shortcut.is_registered(parse_shortcut(shortcut)?))
}

/// Builder for the global shortcut plugin, letting you configure shortcuts and a handler that
/// are registered as soon as the plugin is set up, before [`Builder::build`] is called.
///
/// # Examples
///
/// ```rust,no_run
/// use tauri::Runtime;
///
/// fn setup<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
///     builder.plugin(
///         tauri_plugin_global_shortcut::Builder::new()
///             .with_shortcut("CommandOrControl+Shift+C")
///             .unwrap()
///             .with_handler(|_app, shortcut, event| {
///                 println!("{shortcut:?}: {event:?}");
///             })
///             .build(),
///     )
/// }
/// ```
pub struct Builder<R: Runtime> {
    shortcuts: Vec<Shortcut>,
    handler: Option<HandlerFn<R>>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            shortcuts: Vec::new(),
            handler: Default::default(),
        }
    }
}

impl<R: Runtime> Builder<R> {
    /// Creates a new [`Builder`] with no shortcuts or handler configured.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a shortcut to be registerd.
    pub fn with_shortcut<T>(mut self, shortcut: T) -> Result<Self>
    where
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
    {
        self.shortcuts.push(try_into_shortcut(shortcut)?);
        Ok(self)
    }

    /// Add multiple shortcuts to be registerd.
    pub fn with_shortcuts<S, T>(mut self, shortcuts: S) -> Result<Self>
    where
        S: IntoIterator<Item = T>,
        T: TryInto<ShortcutWrapper>,
        T::Error: std::error::Error,
    {
        for shortcut in shortcuts {
            self.shortcuts.push(try_into_shortcut(shortcut)?);
        }

        Ok(self)
    }

    /// Specify a global shortcut handler that will be triggered for any and all shortcuts.
    pub fn with_handler<F: Fn(&AppHandle<R>, &Shortcut, ShortcutEvent) + Send + Sync + 'static>(
        mut self,
        handler: F,
    ) -> Self {
        self.handler.replace(Box::new(handler));
        self
    }

    /// Builds the plugin. Shortcuts configured with [`Builder::with_shortcut`] /
    /// [`Builder::with_shortcuts`] are registered when the plugin is set up, and the handler
    /// configured with [`Builder::with_handler`] (if any) is called for every hotkey event, in
    /// addition to any handler passed to [`GlobalShortcut::on_shortcut`] /
    /// [`GlobalShortcut::on_shortcuts`] for that specific shortcut.
    pub fn build(self) -> TauriPlugin<R> {
        let handler = self.handler;
        let shortcuts = self.shortcuts;
        PluginBuilder::new("global-shortcut")
            .invoke_handler(tauri::generate_handler![
                register,
                unregister,
                unregister_all,
                is_registered,
            ])
            .setup(move |app, _api| {
                let manager = global_hotkey::GlobalHotKeyManager::new()?;
                let mut store = HashMap::<HotKeyId, RegisteredShortcut<R>>::new();
                for shortcut in shortcuts {
                    manager.register(shortcut)?;
                    store.insert(
                        shortcut.id(),
                        RegisteredShortcut {
                            shortcut,
                            handler: None,
                        },
                    );
                }

                let shortcuts = Arc::new(Mutex::new(store));
                let shortcuts_ = shortcuts.clone();

                let app_handle = app.clone();
                GlobalHotKeyEvent::set_event_handler(Some(move |e: GlobalHotKeyEvent| {
                    if let Some(shortcut) = shortcuts_.lock().unwrap().get(&e.id) {
                        if let Some(handler) = &shortcut.handler {
                            handler(&app_handle, &shortcut.shortcut, e);
                        }
                        if let Some(handler) = &handler {
                            handler(&app_handle, &shortcut.shortcut, e);
                        }
                    }
                }));

                app.manage(GlobalShortcut {
                    app: app.clone(),
                    manager: Arc::new(GlobalHotKeyManager(manager)),
                    shortcuts,
                });
                Ok(())
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shortcut(s: &str) -> Shortcut {
        s.parse().unwrap()
    }

    #[test]
    fn register_all_or_none_registers_everything() {
        let hotkeys = [shortcut("Alt+A"), shortcut("Alt+B")];
        let mut registered = Vec::new();
        let res: std::result::Result<(), ()> = register_all_or_none(
            &hotkeys,
            |h| {
                registered.push(h);
                Ok(())
            },
            |_| panic!("nothing should be unregistered"),
        );
        assert!(res.is_ok());
        assert_eq!(registered, hotkeys);
    }

    #[test]
    fn register_all_or_none_rolls_back_on_failure() {
        let hotkeys = [shortcut("Alt+A"), shortcut("Alt+B"), shortcut("Alt+C")];
        let registered = std::cell::RefCell::new(Vec::new());
        let res = register_all_or_none(
            &hotkeys,
            |h| {
                if h == hotkeys[2] {
                    return Err("taken");
                }
                registered.borrow_mut().push(h);
                Ok(())
            },
            |h| {
                registered.borrow_mut().retain(|r| *r != h);
                Ok(())
            },
        );
        assert_eq!(res, Err("taken"));
        assert!(registered.borrow().is_empty());
    }
}
