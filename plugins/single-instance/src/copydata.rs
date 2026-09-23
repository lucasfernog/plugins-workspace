// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Encoding and decoding of the `WM_COPYDATA` payload the second instance sends to the first one
//! on Windows. Kept platform independent so it can be unit tested everywhere.

/// Decodes the `cwd|arg0|arg1…\0` payload.
///
/// `bytes` is the whole buffer as described by `COPYDATASTRUCT::cbData`, which comes from another
/// process and can't be trusted: it may lack the NUL terminator or be empty. Everything after the
/// first NUL is ignored.
pub fn decode_legacy(bytes: &[u8]) -> (String, Vec<String>) {
    let bytes = bytes
        .iter()
        .position(|b| *b == 0)
        .map_or(bytes, |nul| &bytes[..nul]);
    let data = String::from_utf8_lossy(bytes);
    let mut parts = data.split('|');
    let cwd = parts.next().unwrap_or_default().to_string();
    let args = parts.map(str::to_string).collect();
    (cwd, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_roundtrip() {
        let (cwd, args) = decode_legacy(b"C:\\dir|app.exe|--flag\0");
        assert_eq!(cwd, "C:\\dir");
        assert_eq!(args, vec!["app.exe", "--flag"]);
    }

    #[test]
    fn legacy_without_nul_terminator() {
        let (cwd, args) = decode_legacy(b"C:\\dir|app.exe");
        assert_eq!(cwd, "C:\\dir");
        assert_eq!(args, vec!["app.exe"]);
    }

    #[test]
    fn legacy_ignores_data_after_nul() {
        let (cwd, args) = decode_legacy(b"cwd|a\0garbage|more");
        assert_eq!(cwd, "cwd");
        assert_eq!(args, vec!["a"]);
    }

    #[test]
    fn legacy_empty_and_invalid_utf8() {
        assert_eq!(decode_legacy(b""), (String::new(), Vec::new()));
        let (cwd, args) = decode_legacy(b"c\xffwd|a\xfe");
        assert_eq!(cwd, "c\u{fffd}wd");
        assert_eq!(args, vec!["a\u{fffd}"]);
    }
}
