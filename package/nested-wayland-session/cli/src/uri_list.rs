//! `text/uri-list` payloads for dropped files (RFC 2483 lines of RFC 8089 `file:` URIs).

/// Standard library items.
use std::{ffi::OsStr, os::unix::ffi::OsStrExt, path::Path};

/// Encode a file path as a `file:` URI.
///
/// What:
///      `pub fn file_uri(path: &Path) -> String`.
///  The path's bytes are percent-encoded except ASCII letters,
///  digits,
///  `-`,
///  `.`,
///  `_`,
///  `~`,
///  and the `/` separators,
///  so spaces,
///  `%`,
///  `#`,
///  `?`,
///  line breaks,
///  and every non-ASCII byte survive a client that decodes the URI.
///  A relative path keeps no `/` prefix and is the caller's mistake.
/// Why:
///       `file://` plus the raw path breaks at the first space,
///  and a literal `%20` in a name would be decoded into a space.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fileUri(path: string): string {
///   return "file://" + [...Buffer.from(path)].map(byte => isSafe(byte) ? String.fromCharCode(byte) : "%" + hex(byte)).join("");
/// }
/// ```
///
/// @example
/// ```ts
/// fileUri("/music/a b#1.mp3"); // => "file:///music/a%20b%231.mp3"
/// ```
pub fn file_uri(path: &Path) -> String {
    let mut uri = String::from("file://");
    encode_into(&mut uri, path.as_os_str());
    return uri
}

/// Append the percent-encoded bytes of a path.
///
/// What:
///      `fn encode_into(uri: &mut String, path: &OsStr)`.
/// Why:
///       Keeps the byte loop out of `file_uri`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function encodeInto(uri: { text: string }, path: Buffer): void { ... }
/// ```
fn encode_into(uri: &mut String, path: &OsStr) {
    for byte in path.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            uri.push(char::from(*byte));
        } else {
            uri.push_str(&format!("%{byte:02X}"));
        }
    }
}

/// Build the `text/uri-list` bytes for several files.
///
/// What:
///      `pub fn uri_list(paths: &[&Path]) -> Vec<u8>`.
///  One `file:` URI per line,
///  each ended by CRLF.
/// Why:
///       Dropping several files at once is one list,
///  and the format is the same one a file manager sends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const uriList = (paths: string[]) => Buffer.from(paths.map(p => fileUri(p) + "\r\n").join(""));
/// ```
pub fn uri_list(paths: &[&Path]) -> Vec<u8> {
    let mut list = String::new();
    for path in paths {
        list.push_str(&file_uri(path));
        list.push_str("\r\n");
    }
    return list.into_bytes()
}

/// What:
///      `#[cfg(test)] #[path = "uri_list_tests.rs"] mod tests;`.
/// Why:
///       Keep the encoding tests beside the encoder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./uri_list_tests";
/// ```
#[cfg(test)]
#[path = "uri_list_tests.rs"]
mod tests;
