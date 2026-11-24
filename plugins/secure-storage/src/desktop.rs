// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use keyring_core::{CredentialStore, Entry};
use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

use crate::validate_key;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<SecureStorage<R>> {
    Ok(SecureStorage {
        service: app.config().identifier.clone(),
        store: Mutex::new(None),
        _marker: PhantomData,
    })
}

/// Access to the secure storage APIs.
pub struct SecureStorage<R: Runtime> {
    /// The service the items are stored under: the app identifier.
    service: String,
    /// Connected on first use instead of at startup, so an app still launches when the
    /// credential store is unavailable (e.g. no Secret Service provider on Linux) and a
    /// later call can succeed once it is.
    store: Mutex<Option<Arc<CredentialStore>>>,
    _marker: PhantomData<fn() -> R>,
}

impl<R: Runtime> SecureStorage<R> {
    /// Returns the value stored under `key`, or `None` if there is none.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::Keyring`](crate::Error::Keyring) if the credential store cannot be reached
    /// or read, or the stored value is not valid UTF-8.
    pub fn get(&self, key: &str) -> crate::Result<Option<String>> {
        match self.entry(key)?.get_secret() {
            Ok(value) => Ok(Some(keyring_core::error::decode_password(value)?)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Stores `value` under `key`, replacing any existing value.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::Keyring`](crate::Error::Keyring) if the credential store cannot be reached
    /// or rejects the item, e.g. because it exceeds the platform's size limit
    /// (2560 bytes of UTF-8 on Windows).
    pub fn set(&self, key: &str, value: &str) -> crate::Result<()> {
        // not `set_password`: the Windows store encodes passwords as UTF-16, which
        // halves the size limit for ASCII values. Every store reads the bytes back as UTF-8.
        self.entry(key)?
            .set_secret(value.as_bytes())
            .map_err(Into::into)
    }

    /// Removes the value stored under `key`. Does nothing if there is none.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::Keyring`](crate::Error::Keyring) if the credential store cannot be reached
    /// or fails to delete the item.
    pub fn remove(&self, key: &str) -> crate::Result<()> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn entry(&self, key: &str) -> crate::Result<Entry> {
        validate_key(key)?;
        let store = {
            let mut store = self.store.lock().unwrap();
            match &*store {
                Some(store) => store.clone(),
                None => store.insert(platform_store()?).clone(),
            }
        };
        store.build(&self.service, key, None).map_err(Into::into)
    }
}

fn platform_store() -> keyring_core::Result<Arc<CredentialStore>> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(windows)]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(not(any(target_os = "macos", windows)))]
    let store = zbus_secret_service_keyring_store::Store::new()?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Error;

    fn storage(
        service: &str,
        store: Arc<CredentialStore>,
    ) -> SecureStorage<tauri::test::MockRuntime> {
        SecureStorage {
            service: service.into(),
            store: Mutex::new(Some(store)),
            _marker: PhantomData,
        }
    }

    fn round_trip(storage: &SecureStorage<tauri::test::MockRuntime>) {
        // start clean in case a previous run was interrupted
        storage.remove("token").unwrap();
        assert_eq!(storage.get("token").unwrap(), None);

        storage.set("token", "s3cr3t").unwrap();
        assert_eq!(storage.get("token").unwrap().as_deref(), Some("s3cr3t"));

        storage.set("token", "ünïcödé 🔑").unwrap();
        assert_eq!(storage.get("token").unwrap().as_deref(), Some("ünïcödé 🔑"));
        assert_eq!(storage.get("other").unwrap(), None);

        // the largest value the Windows Credential Manager accepts
        let large = "a".repeat(2560);
        storage.set("token", &large).unwrap();
        assert_eq!(storage.get("token").unwrap(), Some(large));

        storage.remove("token").unwrap();
        assert_eq!(storage.get("token").unwrap(), None);
        // removing a missing item is not an error
        storage.remove("token").unwrap();
    }

    #[test]
    fn round_trip_in_memory() {
        let store = keyring_core::sample::Store::new().unwrap();
        round_trip(&storage("com.tauri.secure-storage.test", store));
    }

    #[test]
    fn services_are_isolated() {
        let store = keyring_core::sample::Store::new().unwrap();
        let a = storage("com.tauri.a", store.clone());
        let b = storage("com.tauri.b", store);
        a.set("token", "a").unwrap();
        assert_eq!(b.get("token").unwrap(), None);
    }

    #[test]
    fn rejects_empty_key() {
        let store = keyring_core::sample::Store::new().unwrap();
        let storage = storage("com.tauri.secure-storage.test", store);
        assert!(matches!(storage.get(""), Err(Error::EmptyKey)));
        assert!(matches!(storage.set("", "value"), Err(Error::EmptyKey)));
        assert!(matches!(storage.remove(""), Err(Error::EmptyKey)));
    }

    /// Talks to the real credential store of the machine, which needs a desktop session
    /// (and may prompt to unlock the keyring), so it only runs on demand:
    /// `cargo test -p tauri-plugin-secure-storage -- --ignored`
    #[test]
    #[ignore]
    fn round_trip_platform_store() {
        round_trip(&storage(
            "com.tauri.secure-storage.test",
            platform_store().unwrap(),
        ));
    }
}
