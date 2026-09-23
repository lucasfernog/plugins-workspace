// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! D-Bus name helpers used on Linux. Kept platform independent so they can be unit tested
//! everywhere.

/// Turns `name` into a valid D-Bus well-known bus name.
///
/// Valid names are returned unchanged. Otherwise characters outside `[A-Za-z0-9_-]` are replaced
/// with `_`, empty elements are dropped and elements starting with a digit are prefixed with `_`
/// (for example a `+` from semver build metadata, or an identifier such as `com.1password.app`).
pub fn bus_name(name: &str) -> String {
    name.split('.')
        .filter(|element| !element.is_empty())
        .map(|element| {
            let element: String = element
                .chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            if element.starts_with(|c: char| c.is_ascii_digit()) {
                format!("_{element}")
            } else {
                element
            }
        })
        .collect::<Vec<_>>()
        .join(".")
}

/// The object path the plugin serves its interface at for the bus name `name` (as returned by
/// [`bus_name`]): `a.b-c.D` becomes `/a/b_c/D`.
pub fn object_path(name: &str) -> String {
    let path: String = name
        .chars()
        .map(|c| match c {
            '.' => '/',
            c if c.is_ascii_alphanumeric() || c == '_' => c,
            _ => '_',
        })
        .collect();
    format!("/{path}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_names_are_unchanged() {
        for name in [
            "com.tauri.dev.SingleInstance",
            "net.my-domain.MyApp.SingleInstance_1_x_x",
            "org.Tauri.SIExampleApp.SingleInstance_1_0_0_beta_1",
        ] {
            assert_eq!(bus_name(name), name);
        }
    }

    #[test]
    fn invalid_names_are_sanitized() {
        assert_eq!(
            bus_name("com.app.SingleInstance_1_0_0_beta_1+build_5"),
            "com.app.SingleInstance_1_0_0_beta_1_build_5"
        );
        assert_eq!(bus_name("com.1password.app"), "com._1password.app");
        assert_eq!(bus_name("com..app."), "com.app");
    }

    #[test]
    fn object_paths() {
        // same as the path derived by previous versions for valid names
        assert_eq!(
            object_path("net.my-domain.MyApp.SingleInstance"),
            "/net/my_domain/MyApp/SingleInstance"
        );
        assert_eq!(
            object_path(&bus_name("com.app.SingleInstance_1_0_0+abc")),
            "/com/app/SingleInstance_1_0_0_abc"
        );
    }
}
