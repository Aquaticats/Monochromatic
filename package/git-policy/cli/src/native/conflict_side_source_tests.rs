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

use super::{copy_located, locate};
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
