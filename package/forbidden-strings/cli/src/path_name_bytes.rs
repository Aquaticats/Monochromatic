//! What:
//!  Native pathname bytes and redacted protocol-safe component spelling.
//! Why:
//!  Candidate scanning must not require every operating-system pathname to be UTF-8.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Keep native path bytes for matching; encode only when constructing the safe display label.
//! ```

/// Import native path components without converting the pathname to a lossy string.
use std::path::{Component, Path};

/// Normalize native separator bytes while retaining every non-separator byte for matching.
pub(crate) fn normalized_path(path: &Path) -> Vec<u8> {
    // Borrow native bytes without display conversion; target semantics are explicit for host-independent controls.
    return normalize_bytes(path.as_os_str().as_encoded_bytes(), cfg!(windows));
}

/// Normalize bytes with explicit target separator semantics so Windows branches are testable on Unix.
pub(crate) fn normalize_bytes(native: &[u8], windows: bool) -> Vec<u8> {
    // Vec owns a normalized copy; the caller's native bytes remain unchanged.
    let mut bytes: Vec<u8> = native.to_vec();
    if windows {
        for byte in &mut bytes {
            if *byte == b'\\' { *byte = b'/'; }
        }
        // A drive-relative name contains a volume prefix and a real first filename component.
        if bytes.len() > 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] != b'/' {
            bytes.insert(2, b'/');
        }
    }
    return bytes;
}

/// Count volume-prefix components excluded by the incumbent pathname rule.
pub(crate) fn prefix_parts(path: &Path) -> usize {
    if !cfg!(windows) { return 0; }
    // Native parsing identifies drive/UNC prefixes; arbitrary colon-containing names remain ordinary components.
    let Some(Component::Prefix(prefix)) = path.components().next() else { return 0; };
    // Native prefix detection stays platform-owned; counting its separator-delimited parts is platform-independent.
    return count_prefix_parts(prefix.as_os_str().as_encoded_bytes());
}

/// Count all parts of an already identified native volume prefix without requiring that platform's Path parser.
///
/// A part is a non-empty byte run between `/` or `\` separators,
///  whatever its bytes,
/// so the `.` of `\\.\COM1` and a `..` share name each count once.
/// The pathname scan consumes exactly this many non-empty components before classifying navigation markers.
/// Windows's parser ends every prefix at a separator or the end of input,
/// and `normalize_bytes` inserts the separator after a drive-relative `C:`,
/// so the last part never merges with the first name.
pub(crate) fn count_prefix_parts(prefix: &[u8]) -> usize {
    let mut count: usize = 0;
    let mut in_component: bool = false;
    // Iterate borrowed bytes once; both native Windows separators terminate a prefix part.
    for byte in prefix {
        if *byte == b'/' || *byte == b'\\' {
            in_component = false;
        } else if !in_component {
            count += 1;
            in_component = true;
        }
    }
    return count;
}

/// Named predicate for the byte-slice component iterator.
pub(crate) fn is_slash(byte: &u8) -> bool {
    return *byte == b'/';
}

/// What:
///  Encode an unmatched component for the existing single-line display protocol.
/// Why:
///  Invalid UTF-8 bytes remain distinguishable and cannot be discarded before matching.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function safeComponent(bytes): string;
/// ```
pub(crate) fn safe_component(component: &[u8]) -> String {
    // String owns the display label; only output spelling, never matcher input, is UTF-8 text.
    let mut safe: String = String::new();
    // Each chunk lends valid text plus the following invalid bytes, so every input byte is accounted for once.
    for chunk in component.utf8_chunks() {
        for character in chunk.valid().chars() {
            if character == ':' {
                safe.push_str("\\x3a");
            } else if character == '\\' {
                safe.push_str("\\\\");
            } else if character.is_control() {
                safe.push_str(format!("\\u{{{:x}}}", character as u32).as_str());
            } else {
                safe.push(character);
            }
        }
        for byte in chunk.invalid() {
            safe.push_str(format!("\\x{byte:02x}").as_str());
        }
    }
    return safe;
}

/// Host-independent controls execute Windows normalization/counting and native display edges.
#[cfg(test)]
#[path = "path_name_bytes_tests.rs"]
mod tests;
