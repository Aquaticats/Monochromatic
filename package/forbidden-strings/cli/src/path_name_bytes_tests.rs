//! What:
//!  Target-independent native separator policy and protocol-safe display controls.
//! Why:
//!  Linux verification must exercise Windows byte decisions without pretending it ran Windows's native Path parser.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Supply target semantics explicitly and assert exact normalized byte sequences.
//! ```

/// Import the pure production policy helpers,
///  not a second normalization implementation.
use super::{normalize_bytes, count_prefix_parts, safe_component};

/// Windows separators normalize only under Windows semantics;
///  native Unix backslashes remain filename bytes.
#[test]
fn target_separator_semantics_are_explicit() {
    // Borrow literal byte slices; compare the owned Vec output to independently written expected bytes.
    assert_eq!(normalize_bytes(b"C:\\folder\\file", true), b"C:/folder/file");
    assert_eq!(normalize_bytes(b"C:\\folder\\file", false), b"C:\\folder\\file");
    assert_eq!(normalize_bytes(b"C:folder\\file", true), b"C:/folder/file");
    assert_eq!(normalize_bytes(b"C:/folder/file", true), b"C:/folder/file");
    assert_eq!(normalize_bytes(b"C:", true), b"C:");
    assert_eq!(normalize_bytes(b"C", true), b"C");
    assert_eq!(normalize_bytes(b"", true), b"");
    assert_eq!(normalize_bytes(b"a:b", true), b"a:/b");
    assert_eq!(normalize_bytes(b"1:b", true), b"1:b");
    assert_eq!(normalize_bytes(b"a-b", true), b"a-b");
}

/// Repeated leading separators do not create fictitious prefix parts or discard UNC server/share names.
#[test]
fn prefix_parts_count_nonempty_runs_with_both_separators() {
    assert_eq!(count_prefix_parts(b"C:"), 1);
    assert_eq!(count_prefix_parts(b"\\\\server\\share"), 2);
    assert_eq!(count_prefix_parts(b"//server/share"), 2);
    assert_eq!(count_prefix_parts(b"\\\\?\\UNC\\server\\share"), 4);
    assert_eq!(count_prefix_parts(b"/\\//"), 0);
    assert_eq!(count_prefix_parts(b""), 0);
}

/// Prefix parts spelled like navigation markers still count,
///  because the scan consumes them as prefix.
#[test]
fn prefix_parts_count_navigation_spellings_and_device_markers() {
    // br"..." keeps backslashes literal; each slice is the raw native prefix std returns for that form.
    assert_eq!(count_prefix_parts(br"\\.\COM1"), 2);
    assert_eq!(count_prefix_parts(br"\\.\C:"), 2);
    assert_eq!(count_prefix_parts(br"\\.\"), 1);
    assert_eq!(count_prefix_parts(br"\\.\.."), 2);
    assert_eq!(count_prefix_parts(br"\\server\.."), 2);
    assert_eq!(count_prefix_parts(br"\\?\a/./b"), 4);
}

/// Invalid bytes,
///  backslashes,
///  control characters and protocol colons have distinguishable safe spellings.
#[test]
fn display_encoding_keeps_adversarial_native_bytes_distinct() {
    assert_eq!(safe_component(b"\xff\xfe"), "\\xff\\xfe");
    assert_eq!(safe_component(b"a:b\\c\t\0\x7f"), "a\\x3ab\\\\c\\u{9}\\u{0}\\u{7f}");
    // as_bytes lends UTF-8 bytes without allocating another String.
    assert_eq!(safe_component("é".as_bytes()), "é");
    assert_eq!(safe_component(b""), "");
}
