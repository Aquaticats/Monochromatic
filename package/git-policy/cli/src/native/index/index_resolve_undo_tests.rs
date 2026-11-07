//! Controls for the resolve-undo extension.

use super::*;

/// The object name used for a side.
fn name(byte: u8) -> Vec<u8> {
    return vec![byte; 20];
}

/// Recording sides,
/// writing and reading back keeps the sides,
/// and the bytes follow `resolve_undo_write`.
#[test]
fn recorded_sides_round_trip_in_git_form() {
    let mut undo: ResolveUndo = ResolveUndo::default();
    undo.record(b"b", 1, 0o100_644, name(1).as_slice());
    undo.record(b"b", 3, 0o100_755, name(3).as_slice());
    undo.record(b"a", 2, 0o100_644, name(2).as_slice());
    // A resolved entry is not recorded.
    undo.record(b"c", 0, 0o100_644, name(9).as_slice());
    let bytes: Vec<u8> = undo.write();
    let mut expected: Vec<u8> = Vec::new();
    expected.extend_from_slice(b"a\0");
    expected.extend_from_slice(b"0\x00100644\x000\x00");
    expected.extend_from_slice(name(2).as_slice());
    expected.extend_from_slice(b"b\0");
    expected.extend_from_slice(b"100644\x000\x00100755\x00");
    expected.extend_from_slice(name(1).as_slice());
    expected.extend_from_slice(name(3).as_slice());
    assert_eq!(bytes, expected);
    assert_eq!(ResolveUndo::read(bytes.as_slice(), 20), Some(undo));
}

/// A mode that is not octal text,
/// and a record cut short,
/// are what Git calls invalid resolve-undo information.
#[test]
fn damaged_records_are_not_read() {
    assert_eq!(ResolveUndo::read(b"a\x00zz\x000\x000\x00", 20), None);
    assert_eq!(ResolveUndo::read(b"a\x00100644\x000\x00", 20), None);
    assert_eq!(ResolveUndo::read(b"a\x00100644\x000\x000\x00short", 20), None);
}
