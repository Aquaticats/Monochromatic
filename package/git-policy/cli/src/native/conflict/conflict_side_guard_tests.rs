//! What:
//!  The paths a command line names, read by their literal spelling,
//! and which of them the index marks as unmerged.
//! Why:
//!  The refusal must never claim a file the command would not replace,
//! so every token that is not one plain file below the working directory adds no name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(unmergedAmong(['a.rs', '*.rs', ':(icase)a.rs'], '', root, new Set(['a.rs']))).toEqual(['a.rs']);
//! ```

use super::{literal_path, unmerged_among};
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;

/// The tokens as the wrapper carries them.
fn tokens(items: &[&str]) -> Vec<OsString> {
    return items.iter().map(|item| { return OsString::from(item); }).collect();
}

/// The conflicted set of a repository whose unmerged entries are `paths`.
fn unmerged(paths: &[&str]) -> BTreeSet<Vec<u8>> {
    return paths.iter().map(|path| { return path.as_bytes().to_vec(); }).collect();
}

/// A literal token names its file below the directory the command runs in,
/// and `.` and `..` are folded.
#[test]
fn a_literal_token_names_one_file_below_the_working_directory() {
    let root: &Path = Path::new("/repo");
    for (prefix, token, expected) in [
        ("", "a.rs", Some("a.rs")),
        ("sub/", "a.rs", Some("sub/a.rs")),
        ("", "./a.rs", Some("a.rs")),
        ("sub/", "../a.rs", Some("a.rs")),
        ("sub/", "../sub/a.rs", Some("sub/a.rs")),
        ("", "/repo/a.rs", Some("a.rs")),
        ("", "/elsewhere/a.rs", None),
        ("", "../a.rs", None),
    ] {
        assert_eq!(
            literal_path(prefix.as_bytes(), root, OsString::from(token).as_os_str()).map(|path| { return String::from_utf8_lossy(path.as_slice()).into_owned(); }),
            expected.map(String::from),
            "{prefix}{token}"
        );
    }
}

/// Every token that is more than one file's name resolves to nothing.
#[test]
fn a_token_that_is_not_one_literal_file_names_nothing() {
    let root: &Path = Path::new("/repo");
    for token in ["", ".", "..", "sub/", "*.rs", "a?.rs", "a[bc].rs", ":(icase)a.rs", ":!a.rs", ":/a.rs"] {
        assert_eq!(literal_path(b"", root, OsString::from(token).as_os_str()), None, "{token}");
    }
}

/// Only the tokens the index marks as unmerged are named,
/// each once,
/// in the order named.
#[test]
fn only_the_unmerged_paths_are_named_once_in_order() {
    let root: &Path = Path::new("/repo");
    let conflicted: BTreeSet<Vec<u8>> = unmerged(&["a.rs", "sub/b.rs"]);
    assert_eq!(
        unmerged_among(tokens(&["a.rs", "sub/b.rs", "a.rs"]).as_slice(), b"", root, &conflicted),
        vec![String::from("a.rs"), String::from("sub/b.rs")]
    );
    assert_eq!(unmerged_among(tokens(&["b.rs"]).as_slice(), b"sub/", root, &conflicted), vec![String::from("sub/b.rs")]);
}

/// A clean file,
/// a directory,
/// a wildcard,
/// pathspec magic and a path outside the worktree are not named,
/// so a refusal never claims them.
#[test]
fn a_path_that_is_not_unmerged_is_not_named() {
    let root: &Path = Path::new("/repo");
    let conflicted: BTreeSet<Vec<u8>> = unmerged(&["a.rs"]);
    for token in ["clean.rs", "sub", "*.rs", ":(icase)a.rs", ":!a.rs", "../a.rs", "/elsewhere/a.rs"] {
        assert!(unmerged_among(tokens(&[token]).as_slice(), b"", root, &conflicted).is_empty(), "{token}");
    }
    assert!(unmerged_among(&[], b"", root, &conflicted).is_empty());
    assert!(unmerged_among(tokens(&["a.rs"]).as_slice(), b"", root, &BTreeSet::new()).is_empty());
}
