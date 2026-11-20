// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::{Arc, OnceLock};

use tauri::{Manager, Runtime, WebviewWindow};
use windows::{
    ApplicationModel::DataTransfer::{DataRequestedEventArgs, DataTransferManager},
    Foundation::{TypedEventHandler, Uri},
    Storage::{IStorageItem, StorageFile},
    Win32::UI::Shell::IDataTransferManagerInterop,
    core::{AgileReference, HSTRING, Interface, Ref, factory},
};
use windows_collections::IIterable;

use crate::{ShareData, ShareOptions};

pub fn share<R: Runtime>(
    window: &WebviewWindow<R>,
    data: ShareData,
    _options: ShareOptions,
) -> crate::Result<()> {
    // the share dialog requires a title
    let title = data.title.clone().unwrap_or_else(|| {
        let app = window.app_handle();
        app.config()
            .product_name
            .clone()
            .unwrap_or_else(|| app.package_info().name.clone())
    });

    let url = data
        .url
        .as_deref()
        .map(|url| Uri::CreateUri(&HSTRING::from(url)))
        .transpose()
        .map_err(|_| crate::Error::InvalidUrl(data.url.clone().unwrap_or_default()))?;

    let files = load_files(&data)?;

    // the share UI must be requested from the thread that owns the window
    crate::desktop::run_on_main_thread(window, move |window| {
        let hwnd = window.hwnd()?;
        let interop = factory::<DataTransferManager, IDataTransferManagerInterop>()?;
        // SAFETY: `hwnd` is a valid top-level window handle owned by this thread.
        let manager: DataTransferManager = unsafe { interop.GetForWindow(hwnd) }?;

        // the handler fills in the data once the share UI asks for it, then unregisters itself
        let token = Arc::new(OnceLock::<i64>::new());
        let token_ = token.clone();
        let handler = TypedEventHandler::new(
            move |manager: Ref<DataTransferManager>, args: Ref<DataRequestedEventArgs>| {
                if let (Some(manager), Some(token)) = (manager.as_ref(), token_.get()) {
                    let _ = manager.RemoveDataRequested(*token);
                }

                let package = args.ok()?.Request()?.Data()?;
                package.Properties()?.SetTitle(&HSTRING::from(&title))?;
                if let Some(text) = &data.text {
                    package.SetText(&HSTRING::from(text))?;
                }
                if let Some(url) = &url {
                    package.SetWebLink(url)?;
                }
                if !files.is_empty() {
                    let items = files
                        .iter()
                        .map(|file| file.resolve()?.cast::<IStorageItem>().map(Some))
                        .collect::<windows::core::Result<Vec<_>>>()?;
                    package.SetStorageItemsReadOnly(&IIterable::<IStorageItem>::from(items))?;
                }
                Ok(())
            },
        );
        let _ = token.set(manager.DataRequested(&handler)?);

        // SAFETY: same as above.
        if let Err(e) = unsafe { interop.ShowShareUIForWindow(hwnd) } {
            if let Some(token) = token.get() {
                let _ = manager.RemoveDataRequested(*token);
            }
            return Err(e.into());
        }

        Ok(())
    })
}

/// Opens the files to share. This waits on async Windows Runtime operations, which is not
/// allowed on a UI thread, so it runs on a thread of its own. The files are returned as agile
/// references so they can be handed to the UI thread.
fn load_files(data: &ShareData) -> crate::Result<Vec<AgileReference<StorageFile>>> {
    if data.files.is_empty() {
        return Ok(Vec::new());
    }

    let paths = data
        .files
        .iter()
        .map(std::path::absolute)
        .collect::<std::io::Result<Vec<_>>>()?;

    std::thread::spawn(move || {
        paths
            .iter()
            .map(|path| {
                let file =
                    StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))?.join()?;
                AgileReference::new(&file)
            })
            .collect::<windows::core::Result<Vec<_>>>()
    })
    .join()
    .map_err(|_| crate::Error::MainThread)?
    .map_err(Into::into)
}
