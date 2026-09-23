// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};

use tauri::{ipc::ScopeObject, utils::acl::Value, AppHandle, Manager, Runtime};

use crate::{scope_entry::EntryRaw, Error};

pub use crate::scope_entry::Application;

#[derive(Debug)]
pub enum Entry {
    Url {
        url: glob::Pattern,
        app: Application,
    },
    Path {
        path: Option<PathBuf>,
        app: Application,
    },
}

impl ScopeObject for Entry {
    type Error = Error;

    fn deserialize<R: Runtime>(
        app_handle: &AppHandle<R>,
        raw: Value,
    ) -> std::result::Result<Self, Self::Error> {
        serde_json::from_value(raw.into())
            .and_then(|raw| {
                let entry = match raw {
                    EntryRaw::Url { url, app } => Entry::Url {
                        url: glob::Pattern::new(&url)
                            .map_err(|e| serde::de::Error::custom(e.to_string()))?,
                        app,
                    },
                    EntryRaw::Path { path, app } => {
                        let path = match app_handle.path().parse(path) {
                            Ok(path) => Some(path),
                            #[cfg(not(target_os = "android"))]
                            Err(tauri::Error::UnknownPath) => None,
                            Err(err) => return Err(serde::de::Error::custom(err.to_string())),
                        };

                        Entry::Path { path, app }
                    }
                };

                Ok(entry)
            })
            .map_err(Into::into)
    }
}

impl Application {
    fn matches(&self, a: Option<&str>) -> bool {
        match self {
            Self::Default => a.is_none(),
            Self::Enable(enable) => *enable,
            Self::App(program) => Some(program.as_str()) == a,
        }
    }
}

impl Entry {
    fn path(&self) -> Option<PathBuf> {
        match self {
            Self::Url { .. } => None,
            Self::Path { path, .. } => path.clone(),
        }
    }

    fn matches_url(&self, u: &str, a: Option<&str>) -> bool {
        match self {
            Self::Url { url, app } => url.matches(u) && app.matches(a),
            Self::Path { .. } => false,
        }
    }

    /// Returns the path pattern of this entry if it is a path entry whose `app` allows opening with `a`.
    fn path_for_program(&self, a: Option<&str>) -> Option<PathBuf> {
        match self {
            Self::Url { .. } => None,
            Self::Path { path, app } if app.matches(a) => path.clone(),
            Self::Path { .. } => None,
        }
    }
}

/// Collects the path patterns of the `allowed` entries that permit opening with the program `with`.
///
/// The path and the program must be allowed by the **same** entry, so a path allowed with the
/// default application does not become openable with a program that only another entry allows.
fn allowed_paths_for_program<'a>(
    allowed: impl IntoIterator<Item = &'a Entry>,
    with: Option<&str>,
) -> Vec<PathBuf> {
    allowed
        .into_iter()
        .filter_map(|e| e.path_for_program(with))
        .collect()
}

#[derive(Debug)]
pub struct Scope<'a, R: Runtime, M: Manager<R>> {
    allowed: Vec<&'a Arc<Entry>>,
    denied: Vec<&'a Arc<Entry>>,
    manager: &'a M,
    _marker: PhantomData<R>,
}

impl<'a, R: Runtime, M: Manager<R>> Scope<'a, R, M> {
    pub(crate) fn new(
        manager: &'a M,
        allowed: Vec<&'a Arc<Entry>>,
        denied: Vec<&'a Arc<Entry>>,
    ) -> Self {
        Self {
            manager,
            allowed,
            denied,
            _marker: PhantomData,
        }
    }

    pub fn is_url_allowed(&self, url: &str, with: Option<&str>) -> bool {
        let denied = self.denied.iter().any(|e| e.matches_url(url, with));
        if denied {
            false
        } else {
            self.allowed.iter().any(|e| e.matches_url(url, with))
        }
    }

    pub fn is_path_allowed(&self, path: &Path, with: Option<&str>) -> crate::Result<bool> {
        let allow = allowed_paths_for_program(self.allowed.iter().map(|e| e.as_ref()), with);
        if allow.is_empty() {
            return Ok(false);
        }

        let fs_scope = tauri::fs::Scope::new(
            self.manager,
            &tauri::utils::config::FsScope::Scope {
                allow,
                deny: self.denied.iter().filter_map(|e| e.path()).collect(),
                require_literal_leading_dot: self
                    .manager
                    .state::<crate::Opener<R>>()
                    .require_literal_leading_dot,
            },
        )?;

        Ok(fs_scope.is_allowed(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_entry(path: &str, app: Application) -> Entry {
        Entry::Path {
            path: Some(PathBuf::from(path)),
            app,
        }
    }

    #[test]
    fn path_and_program_must_come_from_the_same_entry() {
        let entries = [
            path_entry("/downloads/**", Application::Default),
            path_entry("/config/app.toml", Application::App("notepad".into())),
            path_entry("/media/**", Application::Enable(true)),
            Entry::Path {
                path: None,
                app: Application::Enable(true),
            },
            Entry::Url {
                url: glob::Pattern::new("https://*").unwrap(),
                app: Application::Enable(true),
            },
        ];

        assert_eq!(
            allowed_paths_for_program(&entries, None),
            vec![PathBuf::from("/downloads/**"), PathBuf::from("/media/**")]
        );
        assert_eq!(
            allowed_paths_for_program(&entries, Some("notepad")),
            vec![
                PathBuf::from("/config/app.toml"),
                PathBuf::from("/media/**")
            ]
        );
        assert_eq!(
            allowed_paths_for_program(&entries, Some("powershell")),
            vec![PathBuf::from("/media/**")]
        );
    }

    #[test]
    fn app_false_never_matches() {
        let entries = [path_entry("/downloads/**", Application::Enable(false))];
        assert!(allowed_paths_for_program(&entries, None).is_empty());
        assert!(allowed_paths_for_program(&entries, Some("notepad")).is_empty());
    }
}
