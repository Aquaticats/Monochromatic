//! Controls for the records of the copies a merge keeps:
//! the round trip of a path that is not text,
//! the words a record is made of,
//! and what a record that cannot be read names.

use super::{KeptCopy, decode_copy, encode_copy, record_path, hex_of};

/// What:
///  A record of one copy comes back with both names whole,
///  including a name that is not text at all.
/// Why:
///   A copy waits under a name made of hashes and beside a path that may hold any
///  byte a file system takes,
/// so a record that lost a byte would name a copy no command could find
///  or a path no command could read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decodeCopy(encodeCopy({ location, aside }))).toEqual({ location, aside });
/// ```
#[test]
fn a_record_comes_back_with_both_names_whole() {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let plain: KeptCopy = KeptCopy {
            location: std::path::PathBuf::from("/repo/dir/a file.txt"),
            aside: std::path::PathBuf::from("/repo/dir/.cli-git-recovery-0123456789abcdef-00000000-0-1"),
        };
        let back: KeptCopy = decode_copy(encode_copy(&plain).as_bytes()).expect("a record of one copy");
        assert_eq!(back.location, plain.location, "the path comes back whole");
        assert_eq!(back.aside, plain.aside, "the private name comes back whole");
        let awkward: std::path::PathBuf = std::path::PathBuf::from(std::ffi::OsString::from_vec(b"/repo/dir/a\nfile \xff.txt".to_vec()));
        let copy: KeptCopy = KeptCopy { location: awkward.clone(), aside: plain.aside.clone() };
        let whole: KeptCopy = decode_copy(encode_copy(&copy).as_bytes()).expect("a record of a name that is not text");
        assert_eq!(whole.location, awkward, "a name that is not text comes back whole");
        return;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let copy: KeptCopy = KeptCopy {
            location: std::path::PathBuf::from("/repo/dir/a file.txt"),
            aside: std::path::PathBuf::from("/repo/dir/.cli-git-recovery-0123456789abcdef-00000000-0-1"),
        };
        let back: KeptCopy = decode_copy(encode_copy(&copy).as_bytes()).expect("a record of one copy");
        assert_eq!(back.location, copy.location, "the path comes back whole");
    }
}

/// What:
///  A record that is not one names no copy,
///  and neither does a name that is not hex.
/// Why:
///   A record a command cannot read must stop nothing,
/// and it must never be read as a path
///  that was never written:
/// the copy it named is still beside its own path,
/// where the
///  person meets it in `git status`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(decodeCopy(Buffer.from('{not a record}'))).toBeUndefined();
/// ```
#[test]
fn a_record_that_is_not_one_names_no_copy() {
    for text in ["", "\n", "{not a record}", "zzzz\nzzzz\n", "0a0\n0a0\n", "0a\n"] {
        assert!(decode_copy(text.as_bytes()).is_none(), "{text:?} names no copy");
    }
}

/// What:
///  The record's own name is a key of the copy's private name,
///  so it stays short enough for the file system whatever the path is.
/// Why:
///   A path may fill the file system's 255-byte component limit,
/// and a record named after it could not exist,
///  which would make the copy unendable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(basename(recordPath(gitDir, aside)).length).toBe(16 + '.copy'.length);
/// ```
#[test]
fn the_record_name_is_a_key_and_not_a_path() {
    let long: std::path::PathBuf = std::path::PathBuf::from(format!("/repo/{}", "a".repeat(255)));
    let named: Option<std::ffi::OsString> = record_path(std::path::Path::new("/repo/.git"), long.as_path()).file_name().map(std::ffi::OsStr::to_os_string);
    let name: String = named.expect("a record name").to_string_lossy().into_owned();
    assert_eq!(name.len(), ".copy".len() + 16, "the name is one key and the suffix: {name}");
    assert!(name.ends_with(".copy"), "the name says what it is: {name}");
    assert_eq!(hex_of(long.as_path()).len(), 255 * 2 + "/repo/".len() * 2, "the hex text covers the whole path");
}
