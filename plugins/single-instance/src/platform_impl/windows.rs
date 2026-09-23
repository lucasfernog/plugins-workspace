// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::SingleInstanceCallback;
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Manager, RunEvent, Runtime,
};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM,
        LRESULT, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT, WPARAM,
    },
    System::{
        DataExchange::COPYDATASTRUCT,
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
    },
    UI::WindowsAndMessaging::{
        self as w32wm, AllowSetForegroundWindow, CreateWindowExW, DefWindowProcW, DestroyWindow,
        FindWindowW, GetWindowThreadProcessId, RegisterClassExW, SendMessageTimeoutW,
        CREATESTRUCTW, GWLP_USERDATA, GWL_STYLE, SMTO_ABORTIFHUNG, WINDOW_LONG_PTR_INDEX,
        WM_COPYDATA, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_OVERLAPPED, WS_POPUP, WS_VISIBLE,
    },
};

use crate::copydata;

/// How long a starting instance waits for the instance holding the mutex to either create its
/// message window or release the mutex.
const FIND_FIRST_INSTANCE_TIMEOUT: Duration = Duration::from_secs(5);

/// How long the second instance waits for the first instance to handle its message (which
/// includes running the callback) before exiting anyway.
const SEND_TIMEOUT_MS: u32 = 10_000;

struct MutexHandle(Mutex<Option<isize>>);

struct TargetWindowHandle(Mutex<Option<isize>>);

struct UserData<R: Runtime> {
    app: AppHandle<R>,
    callback: RefCell<Box<SingleInstanceCallback<R>>>,
    pending: RefCell<VecDeque<(Vec<String>, String)>>,
}

impl<R: Runtime> UserData<R> {
    fn new(app: AppHandle<R>, callback: Box<SingleInstanceCallback<R>>) -> Self {
        Self {
            app,
            callback: RefCell::new(callback),
            pending: Default::default(),
        }
    }

    unsafe fn from_hwnd_raw(hwnd: HWND) -> *mut Self {
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Self
    }

    unsafe fn from_hwnd<'a>(hwnd: HWND) -> Option<&'a Self> {
        Self::from_hwnd_raw(hwnd).as_ref()
    }

    fn run_callback(&self, args: Vec<String>, cwd: String) {
        self.pending.borrow_mut().push_back((args, cwd));
        // If the callback runs a modal loop (e.g. a blocking message dialog), the window proc can
        // be entered again while it is still running. Don't call it re-entrantly: the message is
        // queued above and the outer call runs it once the callback returns.
        let Ok(mut callback) = self.callback.try_borrow_mut() else {
            return;
        };
        loop {
            let next = self.pending.borrow_mut().pop_front();
            let Some((args, cwd)) = next else {
                break;
            };
            callback(&self.app, args, cwd);
        }
    }
}

pub fn init<R: Runtime>(callback: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app, _api| {
            #[allow(unused_mut)]
            let mut id = app.config().identifier.clone();
            #[cfg(feature = "semver")]
            {
                id.push('_');
                id.push_str(semver_compat_string(&app.package_info().version).as_str());
            }

            let class_name = encode_wide(format!("{id}-sic"));
            let window_name = encode_wide(format!("{id}-siw"));
            let mutex_name = encode_wide(format!("{id}-sim"));

            let hmutex =
                unsafe { CreateMutexW(std::ptr::null(), true.into(), mutex_name.as_ptr()) };
            let last_error = unsafe { GetLastError() };

            if hmutex.is_null() {
                tracing::error!(
                    "single_instance: failed to create the mutex (error {last_error}), launching normally"
                );
                return Ok(());
            }

            if last_error == ERROR_ALREADY_EXISTS {
                // Another instance owns the mutex. Its message window may not exist yet (it is
                // still starting up) or anymore (it is shutting down), so keep looking for the
                // window while waiting for the mutex to be released, instead of giving up right
                // away and ending up with two primary instances.
                let deadline = Instant::now() + FIND_FIRST_INSTANCE_TIMEOUT;
                let mut acquired = false;
                loop {
                    let hwnd = unsafe { FindWindowW(class_name.as_ptr(), window_name.as_ptr()) };
                    if !hwnd.is_null() {
                        forward_to_first_instance(hwnd);
                        app.cleanup_before_exit();
                        std::process::exit(0);
                    }

                    match unsafe { WaitForSingleObject(hmutex, 50) } {
                        // The other instance released the mutex (or exited without releasing
                        // it); we now own it and become the first instance.
                        WAIT_OBJECT_0 | WAIT_ABANDONED => {
                            acquired = true;
                            break;
                        }
                        WAIT_TIMEOUT if Instant::now() < deadline => {}
                        _ => break,
                    }
                }

                if !acquired {
                    tracing::warn!(
                        "single_instance: another instance holds the mutex but its window was not found, launching normally"
                    );
                    unsafe { CloseHandle(hmutex) };
                    return Ok(());
                }
            }

            let userdata = Box::into_raw(Box::new(UserData::new(app.clone(), callback)));
            let hwnd = create_event_target_window::<R>(&class_name, &window_name, userdata);
            if hwnd.is_null() {
                let error = unsafe { GetLastError() };
                tracing::error!(
                    "single_instance: failed to create the message window (error {error}), launching normally"
                );
                // Release the mutex so other instances don't wait for a window that never
                // appears. `userdata` is leaked on purpose: if the window got as far as
                // `WM_CREATE`, `WM_DESTROY` already freed it.
                unsafe {
                    ReleaseMutex(hmutex);
                    CloseHandle(hmutex);
                }
                return Ok(());
            }

            app.manage(MutexHandle(Mutex::new(Some(hmutex as _))));
            app.manage(TargetWindowHandle(Mutex::new(Some(hwnd as _))));

            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

/// Hands the current process' arguments and working directory to the first instance's message
/// window.
fn forward_to_first_instance(hwnd: HWND) {
    // Windows lets us bring a window to the front, but not the first instance. Hand that right
    // over before we exit, so focusing a window from the callback works. Windows takes it back if
    // the user switches to another app in the meantime.
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid != 0 {
        unsafe { AllowSetForegroundWindow(pid) };
    }

    let cwd = std::env::current_dir().unwrap_or_default();
    let cwd = cwd.to_str().unwrap_or_default();
    let args = std::env::args().collect::<Vec<String>>();

    // Prefer the NUL separated payload, which keeps arguments containing `|` intact. A first
    // instance running an older version of this plugin doesn't acknowledge it, so fall back to
    // the legacy format then.
    let data = copydata::encode(cwd, &args);
    // No answer (timeout, e.g. because the callback shows a dialog, or a hung first instance)
    // means a first instance that understands the new format may still be handling it, so only
    // resend when an older first instance explicitly answered without acknowledging it.
    let result = send_copydata(hwnd, copydata::NUL_SEPARATED_DATA, &data);
    if result.is_some_and(|result| result != copydata::ACK) {
        let data = copydata::encode_legacy(cwd, &args);
        send_copydata(hwnd, copydata::LEGACY_DATA, &data);
    }
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    // Destroy the window before releasing the mutex: a starting instance that finds the mutex
    // taken looks for the window, and must not find one that is about to go away.
    // The handles are taken out of the state so calling this more than once (manually, then on
    // `RunEvent::Exit`) doesn't close a handle value that may have been reused since.
    if let Some(hwnd) = manager
        .try_state::<TargetWindowHandle>()
        .and_then(|hwnd| hwnd.0.lock().unwrap().take())
    {
        unsafe { DestroyWindow(hwnd as _) };
    }
    if let Some(hmutex) = manager
        .try_state::<MutexHandle>()
        .and_then(|hmutex| hmutex.0.lock().unwrap().take())
    {
        unsafe {
            ReleaseMutex(hmutex as _);
            CloseHandle(hmutex as _);
        }
    }
}

/// Sends `data` to `hwnd` with `WM_COPYDATA` and returns the window procedure's result, or `None`
/// if the first instance didn't answer within [`SEND_TIMEOUT_MS`] or is hung.
fn send_copydata(hwnd: HWND, kind: usize, data: &[u8]) -> Option<isize> {
    let cds = COPYDATASTRUCT {
        dwData: kind,
        cbData: data.len() as _,
        lpData: data.as_ptr() as _,
    };
    let mut result = 0usize;
    let ok = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_COPYDATA,
            0,
            &cds as *const _ as _,
            SMTO_ABORTIFHUNG,
            SEND_TIMEOUT_MS,
            &mut result,
        )
    };
    (ok != 0).then_some(result as isize)
}

unsafe extern "system" fn single_instance_window_proc<R: Runtime>(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let create_struct = &*(lparam as *const CREATESTRUCTW);
            let userdata = create_struct.lpCreateParams as *const UserData<R>;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, userdata as _);
            0
        }

        WM_COPYDATA => {
            // Any process on the same desktop can send us this message, so don't trust its
            // content: check the pointers and only read `cbData` bytes.
            let cds_ptr = lparam as *const COPYDATASTRUCT;
            if cds_ptr.is_null() {
                return 0;
            }
            let cds = &*cds_ptr;
            if cds.dwData != copydata::NUL_SEPARATED_DATA && cds.dwData != copydata::LEGACY_DATA {
                return 1;
            }
            let bytes: &[u8] = if cds.lpData.is_null() || cds.cbData == 0 {
                &[]
            } else {
                std::slice::from_raw_parts(cds.lpData as *const u8, cds.cbData as usize)
            };
            let (cwd, args) = if cds.dwData == copydata::NUL_SEPARATED_DATA {
                copydata::decode(bytes)
            } else {
                copydata::decode_legacy(bytes)
            };

            if let Some(userdata) = UserData::<R>::from_hwnd(hwnd) {
                userdata.run_callback(args, cwd);
            }

            if cds.dwData == copydata::NUL_SEPARATED_DATA {
                copydata::ACK
            } else {
                1
            }
        }

        WM_DESTROY => {
            let userdata = UserData::<R>::from_hwnd_raw(hwnd);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            // If the window is destroyed from within the callback (e.g. the callback calls
            // `destroy()`), the user data is still in use: leak it instead of freeing it.
            if !userdata.is_null() && (*userdata).callback.try_borrow_mut().is_ok() {
                drop(Box::from_raw(userdata));
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn create_event_target_window<R: Runtime>(
    class_name: &[u16],
    window_name: &[u16],
    userdata: *const UserData<R>,
) -> HWND {
    unsafe {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(single_instance_window_proc::<R>),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(std::ptr::null()),
            hIcon: std::ptr::null_mut(),
            hCursor: std::ptr::null_mut(),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        if RegisterClassExW(&class) == 0 {
            let error = GetLastError();
            // Not fatal on its own when the class already exists; `CreateWindowExW` reports
            // whether we can actually create the window.
            if error != ERROR_CLASS_ALREADY_EXISTS {
                tracing::error!(
                    "single_instance: failed to register the window class (error {error})"
                );
            }
        }

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE
            | WS_EX_TRANSPARENT
            | WS_EX_LAYERED
            // WS_EX_TOOLWINDOW prevents this window from ever showing up in the taskbar, which
            // we want to avoid. If you remove this style, this window won't show up in the
            // taskbar *initially*, but it can show up at some later point. This can sometimes
            // happen on its own after several hours have passed, although this has proven
            // difficult to reproduce. Alternatively, it can be manually triggered by killing
            // `explorer.exe` and then starting the process back up.
            // It is unclear why the bug is triggered by waiting for several hours.
            | WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            window_name.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            GetModuleHandleW(std::ptr::null()),
            userdata as _,
        );
        if hwnd.is_null() {
            return hwnd;
        }
        SetWindowLongPtrW(
            hwnd,
            GWL_STYLE,
            // The window technically has to be visible to receive WM_PAINT messages (which are used
            // for delivering events during resizes), but it isn't displayed to the user because of
            // the LAYERED style.
            (WS_VISIBLE | WS_POPUP) as isize,
        );
        hwnd
    }
}

pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    std::os::windows::prelude::OsStrExt::encode_wide(string.as_ref())
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongW(hwnd, index, value as _) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    w32wm::SetWindowLongPtrW(hwnd, index, value)
}

#[cfg(target_pointer_width = "32")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongW(hwnd, index) as _
}

#[cfg(target_pointer_width = "64")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    w32wm::GetWindowLongPtrW(hwnd, index)
}
