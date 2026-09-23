// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Encoding and decoding of the `WM_COPYDATA` payload the second instance sends to the first one
//! on Windows. Kept platform independent so it can be unit tested everywhere.
//!
//! Two formats exist, told apart by `COPYDATASTRUCT::dwData`:
//!
//! - [`LEGACY_DATA`]: `cwd|arg0|arg1…\0`, sent by plugin versions up to 2.4.5. Arguments that
//!   contain `|` can't be represented.
//! - [`NUL_SEPARATED_DATA`]: `cwd\0arg0\0arg1…`, which can carry any argument since command line
//!   arguments can't contain NUL. A first instance that understands it answers with [`ACK`]; older
//!   first instances answer `1` without handling it, so the second instance then falls back to
//!   the legacy format. This keeps different app versions that run side by side compatible.

/// `dwData` of the legacy `|` separated payload.
pub const LEGACY_DATA: usize = 1542;
/// `dwData` of the NUL separated payload.
pub const NUL_SEPARATED_DATA: usize = 1543;
/// Value the first instance returns from `WM_COPYDATA` after handling a [`NUL_SEPARATED_DATA`]
/// payload.
pub const ACK: isize = 0x5349_4e53;

/// Encodes the NUL separated payload.
pub fn encode(cwd: &str, args: &[String]) -> Vec<u8> {
    let mut data = cwd.as_bytes().to_vec();
    for arg in args {
        data.push(0);
        data.extend_from_slice(arg.as_bytes());
    }
    data
}

/// Decodes the NUL separated payload. `bytes` comes from another process; invalid UTF-8 is
/// converted lossily.
pub fn decode(bytes: &[u8]) -> (String, Vec<String>) {
    let mut parts = bytes
        .split(|b| *b == 0)
        .map(|part| String::from_utf8_lossy(part).into_owned());
    let cwd = parts.next().unwrap_or_default();
    (cwd, parts.collect())
}

/// Encodes the legacy `|` separated payload, for first instances that don't understand
/// [`NUL_SEPARATED_DATA`].
pub fn encode_legacy(cwd: &str, args: &[String]) -> Vec<u8> {
    format!("{cwd}|{}\0", args.join("|")).into_bytes()
}

/// Decodes the legacy `cwd|arg0|arg1…\0` payload.
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

    fn strings(s: &[&str]) -> Vec<String> {
        s.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn roundtrip_keeps_pipes() {
        let args = strings(&["app.exe", "--filter=a|b", "myapp://x?q=1|2", ""]);
        let (cwd, decoded) = decode(&encode("C:\\a|b", &args));
        assert_eq!(cwd, "C:\\a|b");
        assert_eq!(decoded, args);
    }

    #[test]
    fn roundtrip_no_args() {
        assert_eq!(decode(&encode("cwd", &[])), ("cwd".to_string(), vec![]));
        assert_eq!(decode(b""), (String::new(), vec![]));
    }

    #[test]
    fn decode_invalid_utf8() {
        assert_eq!(
            decode(b"c\xff\0a"),
            ("c\u{fffd}".to_string(), strings(&["a"]))
        );
    }

    #[test]
    fn legacy_encode_matches_decode() {
        let args = strings(&["app.exe", "--flag"]);
        assert_eq!(
            decode_legacy(&encode_legacy("C:\\dir", &args)),
            ("C:\\dir".to_string(), args)
        );
    }

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
