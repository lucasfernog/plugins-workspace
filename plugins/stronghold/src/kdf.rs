// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Key derivation used to turn the user password into the key that encrypts a snapshot.
//!
//! Only available when the **kdf** Cargo feature is enabled, which is the case by default.

use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
};

/// NOTE: Hash supplied to Stronghold must be 32bits long.
/// This is a current limitation of Stronghold.
const HASH_LENGTH: usize = 32;

/// Password hashing functions that can be used as the key derivation function of
/// [`Builder::new`](crate::Builder::new).
pub struct KeyDerivation {}

impl KeyDerivation {
    /// Hashes `password` with Argon2 using the salt stored in `salt_path`, returning the
    /// 32 bytes key used to encrypt a snapshot.
    ///
    /// The salt is read from `salt_path` when that file already exists, otherwise a new
    /// random salt is generated and written to it.
    ///
    /// # Panics
    ///
    /// Panics when the salt file cannot be read or written, when its contents are not
    /// 32 bytes long, or when hashing the password fails.
    pub fn argon2(password: &str, salt_path: &Path) -> Vec<u8> {
        try_argon2(password, salt_path).expect("failed to derive the stronghold key")
    }
}

/// Fallible version of [`KeyDerivation::argon2`], used by the plugin so that a salt file
/// that cannot be read or written rejects `Stronghold.load` instead of panicking.
pub(crate) fn try_argon2(password: &str, salt_path: &Path) -> Result<Vec<u8>> {
    let mut salt = [0u8; HASH_LENGTH];
    create_or_get_salt(&mut salt, salt_path)?;

    argon2::hash_raw(password.as_bytes(), &salt, &Default::default())
        .map_err(|e| Error::other(format!("failed to hash the stronghold password: {e}")))
}

fn with_context(e: Error, action: &str, salt_path: &Path) -> Error {
    Error::new(
        e.kind(),
        format!(
            "failed to {action} the stronghold salt file {}: {e}",
            salt_path.display()
        ),
    )
}

fn create_or_get_salt(salt: &mut [u8; HASH_LENGTH], salt_path: &Path) -> Result<()> {
    if salt_path.is_file() {
        return read_salt(salt, salt_path);
    }

    // Generate new salt
    let mut gen = ChaCha20Rng::from_os_rng();
    gen.fill_bytes(salt);
    if let Some(parent) = salt_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| with_context(e, "create the directory of", salt_path))?;
    }
    std::fs::write(salt_path, salt).map_err(|e| with_context(e, "write", salt_path))
}

fn read_salt(salt: &mut [u8; HASH_LENGTH], salt_path: &Path) -> Result<()> {
    let contents = std::fs::read(salt_path).map_err(|e| with_context(e, "read", salt_path))?;
    if contents.len() != HASH_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "the stronghold salt file {} must be {HASH_LENGTH} bytes long, found {} bytes",
                salt_path.display(),
                contents.len()
            ),
        ));
    }
    salt.copy_from_slice(&contents);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tauri-plugin-stronghold-kdf-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn creates_the_salt_and_its_parent_directory() {
        let dir = temp_dir("create");
        let salt_path = dir.join("nested").join("salt.txt");

        let key = try_argon2("password", &salt_path).unwrap();
        assert_eq!(key.len(), HASH_LENGTH);
        assert_eq!(std::fs::read(&salt_path).unwrap().len(), HASH_LENGTH);

        // the stored salt is reused, so the same password derives the same key
        assert_eq!(try_argon2("password", &salt_path).unwrap(), key);
        assert_ne!(try_argon2("other", &salt_path).unwrap(), key);

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_a_salt_file_of_the_wrong_size() {
        let dir = temp_dir("size");
        std::fs::create_dir_all(&dir).unwrap();
        let salt_path = dir.join("salt.txt");
        std::fs::write(&salt_path, [1u8; 5]).unwrap();

        let error = try_argon2("password", &salt_path).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        // the invalid salt is left untouched
        assert_eq!(std::fs::read(&salt_path).unwrap(), [1u8; 5]);

        std::fs::remove_dir_all(dir).unwrap();
    }
}
