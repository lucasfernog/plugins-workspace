// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2::{AnyThread, rc::Retained, runtime::AnyObject};
use objc2_app_kit::{NSSharingServicePicker, NSView};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSRectEdge, NSSize, NSString, NSURL};
use tauri::{Runtime, WebviewWindow};

use crate::{Error, ShareData, ShareOptions};

pub fn share<R: Runtime>(
    window: &WebviewWindow<R>,
    data: ShareData,
    options: ShareOptions,
) -> crate::Result<()> {
    // AppKit views must be used from the main thread
    crate::desktop::run_on_main_thread(window, move |window| show(window, data, options))
}

fn show<R: Runtime>(
    window: &WebviewWindow<R>,
    data: ShareData,
    options: ShareOptions,
) -> crate::Result<()> {
    let mut items: Vec<Retained<AnyObject>> = Vec::new();
    if let Some(text) = &data.text {
        items.push(NSString::from_str(text).into());
    }
    if let Some(url) = &data.url {
        let url = NSURL::URLWithString(&NSString::from_str(url))
            .ok_or_else(|| Error::InvalidUrl(url.clone()))?;
        items.push(url.into());
    }
    for file in &data.files {
        let path = std::path::absolute(file)?;
        let path = NSString::from_str(&path.to_string_lossy());
        items.push(NSURL::fileURLWithPath(&path).into());
    }
    let items = NSArray::from_retained_slice(&items);

    // SAFETY: the pointer is the window's content view, which is alive while the window is,
    // and we are on the main thread.
    let view: &NSView = unsafe { &*window.ns_view()?.cast::<NSView>() };
    let bounds = view.bounds();
    let flipped = view.isFlipped();
    let rect = match options.anchor {
        // the anchor has a top-left origin, AppKit views have a bottom-left one unless flipped
        Some(anchor) => NSRect::new(
            NSPoint::new(
                anchor.x,
                if flipped {
                    anchor.y
                } else {
                    bounds.size.height - anchor.y - anchor.height
                },
            ),
            NSSize::new(anchor.width, anchor.height),
        ),
        None => NSRect::new(
            NSPoint::new(bounds.size.width / 2., bounds.size.height / 2.),
            NSSize::new(0., 0.),
        ),
    };
    // show the picker below the anchor
    let edge = if flipped {
        NSRectEdge::MaxY
    } else {
        NSRectEdge::MinY
    };

    // SAFETY: all the items are NSString or NSURL, which the picker accepts.
    let picker =
        unsafe { NSSharingServicePicker::initWithItems(NSSharingServicePicker::alloc(), &items) };
    picker.showRelativeToRect_ofView_preferredEdge(rect, view, edge);

    Ok(())
}
