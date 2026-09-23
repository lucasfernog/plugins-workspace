// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/pfernie/reqwest_cookie_store/blob/2ec4afabcd55e24d3afe3f0626ee6dc97bed938d/src/lib.rs

use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::Receiver,
        Arc, Mutex,
    },
};

use cookie_store::{CookieStore, RawCookie, RawCookieParseError};
use reqwest::header::HeaderValue;

fn set_cookies(
    cookie_store: &mut CookieStore,
    cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
    url: &url::Url,
) {
    let cookies = cookie_headers.filter_map(|val| {
        std::str::from_utf8(val.as_bytes())
            .map_err(RawCookieParseError::from)
            .and_then(RawCookie::parse)
            .map(|c| c.into_owned())
            .ok()
    });
    cookie_store.store_response_cookies(cookies, url);
}

fn cookies(cookie_store: &CookieStore, url: &url::Url) -> Option<HeaderValue> {
    let s = cookie_store
        .get_request_values(url)
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ");

    if s.is_empty() {
        return None;
    }

    HeaderValue::from_maybe_shared(bytes::Bytes::from(s)).ok()
}

fn cookies_to_str(cookie_store: &CookieStore) -> Result<String, serde_json::Error> {
    let cookies = cookie_store
        .iter_unexpired()
        .filter(|cookie| cookie.is_persistent())
        .collect::<Vec<_>>();
    serde_json::to_string(&cookies)
}

/// Writes `contents` to a temporary file next to `path`, then renames it to `path`, so a crash
/// in the middle of the write never leaves a truncated file behind.
fn write_atomically(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    std::fs::write(&tmp, contents)?;
    std::fs::rename(&tmp, path)
}

/// Orders the writes of the cookie jar snapshots.
#[derive(Debug, Default)]
struct SaveState {
    /// Generation of the latest snapshot taken.
    requested: AtomicU64,
    /// Generation of the latest snapshot written to disk. Its lock is held while writing, so
    /// writes never overlap.
    written: Mutex<u64>,
}

/// A [`cookie_store::CookieStore`] wrapped internally by a [`std::sync::Mutex`], suitable for use in
/// async/concurrent contexts.
#[derive(Debug)]
pub struct CookieStoreMutex {
    pub path: PathBuf,
    store: Mutex<CookieStore>,
    save_state: Arc<SaveState>,
}

impl CookieStoreMutex {
    /// Create a new [`CookieStoreMutex`] from an existing [`cookie_store::CookieStore`].
    pub fn new(path: PathBuf, cookie_store: CookieStore) -> CookieStoreMutex {
        CookieStoreMutex {
            path,
            store: Mutex::new(cookie_store),
            save_state: Default::default(),
        }
    }

    pub fn load<R: std::io::BufRead>(
        path: PathBuf,
        reader: R,
    ) -> cookie_store::Result<CookieStoreMutex> {
        cookie_store::serde::load(reader, |c| serde_json::from_str(c))
            .map(|store| CookieStoreMutex::new(path, store))
    }

    /// Persists a snapshot of the persistent cookies in the background.
    ///
    /// Writes are serialized and a snapshot is skipped when a newer one was already written, so
    /// the file always ends up with the latest snapshot. The returned receiver gets a message
    /// once the file holds this snapshot or a newer one.
    pub fn request_save(&self) -> cookie_store::Result<Receiver<()>> {
        let (generation, cookie_str) = {
            let store = self.store.lock().expect("poisoned cookie jar mutex");
            // taken under the store lock, so generations follow the order of the snapshots
            let generation = self.save_state.requested.fetch_add(1, Ordering::SeqCst) + 1;
            (generation, cookies_to_str(&store)?)
        };
        let path = self.path.clone();
        let save_state = self.save_state.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        tauri::async_runtime::spawn_blocking(move || {
            let mut written = save_state.written.lock().unwrap_or_else(|e| e.into_inner());
            if *written < generation {
                if let Err(_e) = write_atomically(&path, cookie_str.as_bytes()) {
                    #[cfg(feature = "tracing")]
                    tracing::error!("failed to save cookie jar: {_e}");
                    return;
                }
                *written = generation;
            }
            let _ = tx.send(());
        });
        Ok(rx)
    }
}

impl reqwest::cookie::CookieStore for CookieStoreMutex {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        set_cookies(&mut self.store.lock().unwrap(), cookie_headers, url);

        // try to persist cookies immediately asynchronously
        if let Err(_e) = self.request_save() {
            #[cfg(feature = "tracing")]
            tracing::error!("failed to save cookie jar: {_e}");
        }
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.store.lock().unwrap();
        cookies(&store, url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_are_atomic_and_keep_the_latest_snapshot() {
        let dir = std::env::temp_dir().join(format!(
            "tauri-plugin-http-cookies-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".cookies");

        let jar = CookieStoreMutex::new(path.clone(), Default::default());
        let url = url::Url::parse("https://tauri.app").unwrap();
        // every response setting a cookie requests a save in the background
        for i in 0..20 {
            let header = HeaderValue::from_str(&format!("cookie{i}=value; Max-Age=3600")).unwrap();
            reqwest::cookie::CookieStore::set_cookies(&jar, &mut std::iter::once(&header), &url);
        }
        jar.request_save().unwrap().recv().unwrap();

        let loaded = CookieStoreMutex::load(
            path.clone(),
            std::io::BufReader::new(std::fs::File::open(&path).unwrap()),
        )
        .unwrap();
        let count = loaded.store.lock().unwrap().iter_unexpired().count();
        assert_eq!(count, 20);

        let mut tmp = path.as_os_str().to_owned();
        tmp.push(".tmp");
        assert!(!PathBuf::from(tmp).exists());

        std::fs::remove_dir_all(dir).unwrap();
    }
}
