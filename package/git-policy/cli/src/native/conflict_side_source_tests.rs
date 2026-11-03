//! What:
//!  Finding and copying the worktree file of a conflicted path.
//! Why:
//!  A copy that is stale, torn or taken from the wrong object would let the command that follows destroy the only
//!  current version of a half-resolved file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(locate(root, 'a.rs')).toBeDefined(); expect(locate(root, 'gone.rs')).toBeUndefined();
//! ```

use super::{SaveFailure, copy_located, locate};
use crate::test_support::{fixture, remove};

/// A file that is there is found and copied whole; a path with nothing there is not found.
#[test]
fn a_present_file_is_located_and_copied() {
    let base = fixture("source-present");
    std::fs::write(base.join("a.rs"), b"half \xff resolved\n").expect("a");
    let located = locate(base.as_path(), b"a.rs").expect("lookup").expect("present");
    copy_located(&located, base.join("copy").as_path(), &mut || {}).expect("copy");
    assert_eq!(std::fs::read(base.join("copy")).expect("copy"), b"half \xff resolved\n");
    assert!(locate(base.as_path(), b"gone.rs").expect("lookup").is_none());
    remove(base.as_path());
}

/// A link or a file in front of the last name is a failure that names that ancestor; a missing ancestor is nothing to save.
#[test]
fn an_ancestor_that_is_a_link_or_a_file_is_a_failure_naming_it() {
    let base = fixture("source-ancestors");
    std::fs::create_dir_all(base.join("elsewhere")).expect("elsewhere");
    std::fs::write(base.join("elsewhere/file"), b"reached through the link\n").expect("file");
    std::os::unix::fs::symlink("elsewhere", base.join("linked")).expect("link");
    std::os::unix::fs::symlink("nowhere", base.join("dangling")).expect("dangling link");
    std::fs::write(base.join("plain"), b"a file\n").expect("plain file");
    std::fs::create_dir_all(base.join("real/inner")).expect("real directories");
    std::os::unix::fs::symlink("../elsewhere", base.join("real/inner/deep")).expect("deep link");
    for (relative, subject) in [
        ("linked/file", "linked"),
        ("dangling/file", "dangling"),
        ("plain/file", "plain"),
        ("real/inner/deep/file", "real/inner/deep"),
    ] {
        let failure: SaveFailure = locate(base.as_path(), relative.as_bytes()).expect_err(relative);
        assert_eq!(failure.subject, subject, "{relative}: {failure:?}");
        assert!(!failure.reason.is_empty(), "{relative}: {failure:?}");
    }
    assert!(locate(base.as_path(), b"absent/file").expect("a missing ancestor").is_none());
    assert!(locate(base.as_path(), b"real/absent").expect("a missing name").is_none());
    remove(base.as_path());
}
