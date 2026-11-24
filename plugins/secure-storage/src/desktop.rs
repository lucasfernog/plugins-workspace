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
        connect: platform_store,
        store: Mutex::new(None),
        _marker: PhantomData,
    })
}

/// Access to the secure storage APIs.
pub struct SecureStorage<R: Runtime> {
    /// The service the items are stored under: the app identifier.
    service: String,
    /// Connects to the credential store.
    connect: fn() -> keyring_core::Result<Arc<CredentialStore>>,
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
        let value = self.with_entry(key, |entry| match entry.get_secret() {
            Ok(value) => Ok(Some(value)),
            Err(keyring_core::Error::NoEntry) => Ok(None),
            // any of the duplicates: the next `set` or `remove` cleans them up
            Err(keyring_core::Error::Ambiguous(duplicates)) => first_secret(&duplicates),
            Err(e) => Err(e),
        })?;
        Ok(value
            .map(keyring_core::error::decode_password)
            .transpose()?)
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
        self.with_entry(key, |entry| match entry.set_secret(value.as_bytes()) {
            Err(keyring_core::Error::Ambiguous(duplicates)) => {
                delete_all(&duplicates)?;
                entry.set_secret(value.as_bytes())
            }
            result => result,
        })
    }

    /// Removes the value stored under `key`. Does nothing if there is none.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyKey`](crate::Error::EmptyKey) if `key` is empty and
    /// [`Error::Keyring`](crate::Error::Keyring) if the credential store cannot be reached
    /// or fails to delete the item.
    pub fn remove(&self, key: &str) -> crate::Result<()> {
        self.with_entry(key, |entry| match entry.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
            Err(keyring_core::Error::Ambiguous(duplicates)) => delete_all(&duplicates),
            Err(e) => Err(e),
        })
    }

    /// Runs `operation` on the entry of `key`.
    ///
    /// A store can hold a session that dies with the provider: the Secret Service one is
    /// opened when connecting and is gone once GNOME Keyring or KWallet restarts. So when
    /// the platform fails, the operation is retried once on a new connection instead of
    /// failing until the app restarts.
    fn with_entry<T>(
        &self,
        key: &str,
        operation: impl Fn(&Entry) -> keyring_core::Result<T>,
    ) -> crate::Result<T> {
        validate_key(key)?;
        let run = |store: &CredentialStore| {
            store
                .build(&self.service, key, None)
                .and_then(|entry| operation(&entry))
        };
        let store = self.store(None)?;
        match run(&*store) {
            Err(keyring_core::Error::PlatformFailure(_)) => Ok(run(&*self.store(Some(&store))?)?),
            result => Ok(result?),
        }
    }

    /// Returns the cached connection to the credential store, connecting first if there
    /// is none or it is the `stale` one.
    fn store(
        &self,
        stale: Option<&Arc<CredentialStore>>,
    ) -> keyring_core::Result<Arc<CredentialStore>> {
        let mut store = self.store.lock().unwrap();
        // another thread may have replaced the stale connection already
        if let (Some(stale), Some(current)) = (stale, &*store)
            && Arc::ptr_eq(stale, current)
        {
            *store = None;
        }
        match &*store {
            Some(store) => Ok(store.clone()),
            None => Ok(store.insert((self.connect)()?).clone()),
        }
    }
}

// A store can hold several items for one key: the Secret Service matches items by their
// attributes in every collection, so a copy in a second keyring is a duplicate. The entry
// of the key then fails every operation with `Ambiguous`, which hands out the duplicates.

fn first_secret(duplicates: &[Entry]) -> keyring_core::Result<Option<Vec<u8>>> {
    for duplicate in duplicates {
        match duplicate.get_secret() {
            Ok(value) => return Ok(Some(value)),
            // removed in the meantime
            Err(keyring_core::Error::NoEntry) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(None)
}

fn delete_all(duplicates: &[Entry]) -> keyring_core::Result<()> {
    for duplicate in duplicates {
        match duplicate.delete_credential() {
            Ok(()) | Err(keyring_core::Error::NoEntry) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

fn platform_store() -> keyring_core::Result<Arc<CredentialStore>> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(windows)]
    let store = {
        // Credentials are named `{key}@{identifier}`. The store refuses an identifier that
        // contains the divider, so the last `@` always separates the two and no key of one
        // app can name a credential of another. The default divider is `.`, which
        // identifiers contain: `token.com` of `example.app` would be `token` of
        // `com.example.app`.
        let config = [("divider", "@"), ("service_no_divider", "true")];
        windows_native_keyring_store::Store::new_with_configuration(&config.into())?
    };
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
            connect: || keyring_core::sample::Store::new().map(|store| store as _),
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

    /// Stores two items under `token`, as a copy in a second keyring would.
    fn storage_with_duplicates() -> SecureStorage<tauri::test::MockRuntime> {
        let store: Arc<CredentialStore> = keyring_core::sample::Store::new().unwrap();
        for comment in ["first", "second"] {
            let modifiers = [("force-create", comment)].into();
            let entry = store.build("com.tauri.a", "token", Some(&modifiers));
            entry.unwrap();
        }
        let entry = store.build("com.tauri.a", "token", None).unwrap();
        assert!(matches!(
            entry.get_secret(),
            Err(keyring_core::Error::Ambiguous(duplicates)) if duplicates.len() == 2
        ));
        storage("com.tauri.a", store)
    }

    #[test]
    fn reads_a_duplicated_item() {
        // the sample store creates the duplicates with an empty value
        let storage = storage_with_duplicates();
        assert_eq!(storage.get("token").unwrap().as_deref(), Some(""));
    }

    #[test]
    fn set_replaces_a_duplicated_item() {
        let storage = storage_with_duplicates();
        storage.set("token", "s3cr3t").unwrap();
        assert_eq!(storage.get("token").unwrap().as_deref(), Some("s3cr3t"));
        // a single item is left
        storage.remove("token").unwrap();
        assert_eq!(storage.get("token").unwrap(), None);
    }

    #[test]
    fn remove_deletes_a_duplicated_item() {
        let storage = storage_with_duplicates();
        storage.remove("token").unwrap();
        assert_eq!(storage.get("token").unwrap(), None);
    }

    #[test]
    fn reconnects_when_the_platform_fails() {
        let failure = || keyring_core::Error::PlatformFailure("session is closed".into());
        let stale: Arc<CredentialStore> = keyring_core::mock::Store::new().unwrap();
        let entry = stale.build("com.tauri.a", "token", None).unwrap();
        entry.set_secret(b"stale").unwrap();
        let storage = storage("com.tauri.a", stale.clone());

        // the mock fails once; the retry runs on the store `connect` returns, which is empty
        let mock: &keyring_core::mock::Cred = entry.as_any().downcast_ref().unwrap();
        mock.set_error(failure());
        assert_eq!(storage.get("token").unwrap(), None);

        // and that store is kept for the calls that follow
        storage.set("token", "fresh").unwrap();
        assert_eq!(storage.get("token").unwrap().as_deref(), Some("fresh"));
        assert_eq!(entry.get_secret().unwrap(), b"stale");
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
