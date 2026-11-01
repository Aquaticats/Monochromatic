//! What:
//!  Controls for object-name validation and mode mapping.
//! Why:
//!  An accepted object name is written into the reader's request stream unescaped,
//!      so every rejection here is a request-injection guard.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseObjectId(Buffer.from('HEAD'))).toBeUndefined();
//! ```

/// Import the functions and types under test.
use super::{
    CandidateMode, ObjectId, SHA1_HEX_LENGTH, SHA256_HEX_LENGTH, mode_from_git, parse_object_id,
};

/// Complete SHA-1 and SHA-256 names are accepted and printed back unchanged.
#[test]
fn complete_lowercase_names_are_accepted() {
    assert_eq!(SHA1_HEX_LENGTH, 40);
    assert_eq!(SHA256_HEX_LENGTH, 64);
    for text in [
        "0123456789abcdef0123456789abcdef01234567",
        "0000000000000000000000000000000000000000",
        "ffffffffffffffffffffffffffffffffffffffff",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    ] {
        let object: ObjectId = parse_object_id(text.as_bytes()).expect("complete name");
        assert_eq!(object.as_str(), text);
    }
}

/// Every other length,
///  digit case and character is refused,
///  including the bytes next to each accepted range.
#[test]
fn everything_else_is_refused() {
    let valid: &str = "0123456789abcdef0123456789abcdef01234567";
    for text in [
        "",
        "HEAD",
        &valid[..39],
        "0123456789abcdef0123456789abcdef012345678",
        &"a".repeat(63),
        &"a".repeat(65),
        "0123456789ABCDEF0123456789abcdef01234567",
        "0123456789abcdef0123456789abcdef0123456g",
        "0123456789abcdef0123456789abcdef0123456`",
        "0123456789abcdef0123456789abcdef0123456/",
        "0123456789abcdef0123456789abcdef0123456:",
        "0123456789abcdef0123456789abcdef0123456 ",
        "0123456789abcdef0123456789abcdef0123456\n",
        "0123456789abcdef0123456789abcdef012345\n7",
    ] {
        assert_eq!(parse_object_id(text.as_bytes()), None, "{text:?}");
    }
    // A byte that is not ASCII at all is refused without being decoded.
    let mut bytes: Vec<u8> = valid.as_bytes().to_vec();
    bytes[5] = 0xff;
    assert_eq!(parse_object_id(bytes.as_slice()), None);
}

/// Names compare and hash by their text,
///  so one object has one key.
#[test]
fn names_are_usable_as_map_keys() {
    let first: ObjectId = parse_object_id(&[b'a'; 40]).expect("name");
    let same: ObjectId = parse_object_id(&[b'a'; 40]).expect("name");
    let other: ObjectId = parse_object_id(&[b'b'; 40]).expect("name");
    assert_eq!(first, same);
    assert_ne!(first, other);
    let mut seen: std::collections::HashMap<ObjectId, u8> = std::collections::HashMap::new();
    seen.insert(first, 1);
    seen.insert(same, 2);
    seen.insert(other, 3);
    assert_eq!(seen.len(), 2);
}

/// The four candidate modes map exactly;
///  directories,
///  the absent side and near misses do not.
#[test]
fn modes_map_exactly() {
    assert_eq!(mode_from_git(b"100644"), Some(CandidateMode::Regular));
    assert_eq!(mode_from_git(b"100755"), Some(CandidateMode::Executable));
    assert_eq!(mode_from_git(b"120000"), Some(CandidateMode::Symlink));
    assert_eq!(mode_from_git(b"160000"), Some(CandidateMode::Gitlink));
    for text in [
        "040000", "000000", "100664", "100645", "", "100644 ", "0100644", "120001", "160001",
    ] {
        assert_eq!(mode_from_git(text.as_bytes()), None, "{text:?}");
    }
}
