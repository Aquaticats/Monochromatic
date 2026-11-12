//! What:
//!  The two listings that name a path whose content the index holds while the worktree does not,
//!  and the reading that intersects them.
//! Why:
//!  A save step that reads only worktree files reports such a path as having nothing to copy,
//!  and the abort that follows would then discard it unsaved.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(stagedPaths('100644 abc 0\tnew.txt\0')).toEqual(['new.txt']);
//! expect(listingArguments([], '--stage', ['new.txt'])).toContain(':(top,literal)new.txt');
//! ```

use super::{listing_arguments, staged_paths};
use std::ffi::OsString;

/// The words of an argument list,
///  for comparing queries as text.
fn words(arguments: &[OsString]) -> Vec<String> {
    return arguments.iter().map(|argument| { return argument.to_string_lossy().into_owned(); }).collect();
}

/// Each listing names exactly the paths it was given,
///  literally and below the repository root,
///  after the caller's global options.
#[test]
fn each_listing_names_exactly_the_paths_it_was_given() {
    let global: [OsString; 2] = [OsString::from("-C"), OsString::from("/repo")];
    let asked: Vec<String> = words(
        listing_arguments(
            &global,
            "--stage",
            [b"new.txt".to_vec(), b"dir/with space/*.rs".to_vec()].as_slice(),
        )
        .expect("both are paths here")
        .as_slice(),
    );
    assert_eq!(
        asked,
        vec![
            "-C",
            "/repo",
            "ls-files",
            "--stage",
            "-z",
            "--",
            ":(top,literal)new.txt",
            ":(top,literal)dir/with space/*.rs",
        ]
    );
}

/// Stage 0 is the entry an abort resets;
///  an unresolved conflict's stages are not work to copy.
#[test]
fn only_a_stage_zero_entry_names_content_the_index_holds() {
    let answer: &[u8] = b"100644 1111111111111111111111111111111111111111 0\tnew.txt\0\
        100644 2222222222222222222222222222222222222222 1\tconflicted.txt\0\
        100644 3333333333333333333333333333333333333333 2\tconflicted.txt\0\
        100644 4444444444444444444444444444444444444444 0\tdir/with\tspace.txt\0";
    assert_eq!(
        staged_paths(answer),
        vec![b"new.txt".to_vec(), b"dir/with\tspace.txt".to_vec()],
        "a path's own tab stays in the path"
    );
    assert!(staged_paths(b"").is_empty());
    // A record that is not one listing entry names nothing.
    assert!(staged_paths(b"not a listing entry\0").is_empty());
}
