// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, collections::HashMap};

use log::RecordBuilder;

use crate::{LogLevel, WEBVIEW_TARGET};

#[tauri::command]
pub fn log(
    level: LogLevel,
    message: String,
    location: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
    key_values: Option<HashMap<String, String>>,
) {
    let level = log::Level::from(level);

    // The location and file end up in the log record's target and file, so escape control characters such as
    // line breaks to prevent the webview from forging log lines through them.
    let location = location.map(escape_control_chars);
    let location = location.as_deref();
    let file = file.map(escape_control_chars);
    let file = file.as_deref();

    let target = if let Some(location) = location {
        format!("{WEBVIEW_TARGET}::{location}")
    } else {
        WEBVIEW_TARGET.to_owned()
    };

    let mut builder = RecordBuilder::new();
    builder.level(level).target(&target).file(file).line(line);

    let key_values = key_values.unwrap_or_default();
    let mut kv = HashMap::new();
    for (k, v) in key_values.iter() {
        kv.insert(k.as_str(), v.as_str());
    }
    builder.key_values(&kv);
    #[cfg(feature = "tracing")]
    emit_trace(level, &message, location, file, line, &kv);

    log::logger().log(&builder.args(format_args!("{message}")).build());
}

/// Escapes the control characters (e.g. `\n`) in `value` with [`char::escape_default`].
fn escape_control_chars(value: &str) -> Cow<'_, str> {
    if !value.chars().any(char::is_control) {
        return Cow::Borrowed(value);
    }
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_control() {
            escaped.extend(c.escape_default());
        } else {
            escaped.push(c);
        }
    }
    Cow::Owned(escaped)
}

// Target becomes default and location is added as a parameter
#[cfg(feature = "tracing")]
fn emit_trace(
    level: log::Level,
    message: &String,
    location: Option<&str>,
    file: Option<&str>,
    line: Option<u32>,
    kv: &HashMap<&str, &str>,
) {
    macro_rules! emit_event {
        ($level:expr) => {
            tracing::event!(
                target: WEBVIEW_TARGET,
                $level,
                message = %message,
                location = location,
                file,
                line,
                ?kv
            )
        };
    }
    match level {
        log::Level::Error => emit_event!(tracing::Level::ERROR),
        log::Level::Warn => emit_event!(tracing::Level::WARN),
        log::Level::Info => emit_event!(tracing::Level::INFO),
        log::Level::Debug => emit_event!(tracing::Level::DEBUG),
        log::Level::Trace => emit_event!(tracing::Level::TRACE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_control_chars() {
        assert!(matches!(
            escape_control_chars("foo@http://localhost/main.js:1:2"),
            Cow::Borrowed("foo@http://localhost/main.js:1:2")
        ));
        assert_eq!(
            escape_control_chars("a\r\n2026-01-01][my_crate][ERROR] forged\u{1b}[0m"),
            "a\\r\\n2026-01-01][my_crate][ERROR] forged\\u{1b}[0m"
        );
    }
}
