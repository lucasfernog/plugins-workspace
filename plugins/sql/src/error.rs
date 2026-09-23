// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// Errors that can happen while connecting to a database, running migrations
/// or executing a query.
///
/// Serializes to its [`std::fmt::Display`] representation, which is what the
/// frontend receives when a command fails.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error reported by [`sqlx`], such as a failed connection or a query the database rejected.
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    /// A migration registered with [`crate::Builder::add_migrations`] could not be resolved or applied.
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    /// The connection string is missing its `scheme:` prefix, or the scheme does not
    /// match any of the enabled database drivers. Contains the offending connection string.
    ///
    /// The password of the connection string is masked in the error message.
    #[error("invalid connection url: {}", redact_password(.0))]
    InvalidDbUrl(String),
    /// The requested database has not been connected to with the `load` command
    /// and is not listed in the plugin's `preload` configuration.
    /// Contains the connection string of the database.
    ///
    /// The password of the connection string is masked in the error message.
    #[error("database {} not loaded", redact_password(.0))]
    DatabaseNotLoaded(String),
    /// A value selected from the database has a SQL type that cannot be converted
    /// to JSON. Contains the name of that SQL type.
    #[error("unsupported datatype: {0}")]
    UnsupportedDatatype(String),
}

/// Masks the password of a connection string such as
/// `postgres://user:password@host/db` or `mysql://host/db?password=secret`,
/// so it does not end up in error messages sent to the frontend or in logs.
fn redact_password(conn_url: &str) -> String {
    const MASK: &str = "***";
    let mut redacted = conn_url.to_string();

    // `scheme://user:password@host`
    if let Some(start) = redacted.find("://").map(|i| i + 3) {
        let authority_end = redacted[start..]
            .find(['/', '?', '#'])
            .map_or(redacted.len(), |i| start + i);
        if let Some(at) = redacted[start..authority_end].rfind('@') {
            let userinfo = &redacted[start..start + at];
            if let Some(colon) = userinfo.find(':') {
                redacted.replace_range(start + colon + 1..start + at, MASK);
            }
        }
    }

    // `?password=secret` / `&password=secret`
    for key in ["?password=", "&password="] {
        if let Some(start) = redacted.find(key).map(|i| i + key.len()) {
            let end = redacted[start..]
                .find(['&', '#'])
                .map_or(redacted.len(), |i| start + i);
            redacted.replace_range(start..end, MASK);
        }
    }

    redacted
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_are_redacted() {
        for (url, expected) in [
            ("sqlite:test.db", "sqlite:test.db"),
            ("sqlite:user:x@y.db", "sqlite:user:x@y.db"),
            (
                "postgres://user:secret@localhost/db",
                "postgres://user:***@localhost/db",
            ),
            (
                "mysql://user:p@ss@host:3306/db?ssl-mode=required",
                "mysql://user:***@host:3306/db?ssl-mode=required",
            ),
            (
                "postgres://user@localhost/db",
                "postgres://user@localhost/db",
            ),
            ("postgres://localhost/db", "postgres://localhost/db"),
            (
                "postgres://localhost/db?user=me&password=secret&sslmode=disable",
                "postgres://localhost/db?user=me&password=***&sslmode=disable",
            ),
            (
                "mysql://host/db?password=secret",
                "mysql://host/db?password=***",
            ),
        ] {
            assert_eq!(redact_password(url), expected, "{url}");
        }
    }

    #[test]
    fn error_messages_do_not_contain_passwords() {
        let url = "postgres://user:secret@localhost/db".to_string();
        assert_eq!(
            Error::DatabaseNotLoaded(url.clone()).to_string(),
            "database postgres://user:***@localhost/db not loaded"
        );
        assert_eq!(
            Error::InvalidDbUrl(url).to_string(),
            "invalid connection url: postgres://user:***@localhost/db"
        );
    }
}
