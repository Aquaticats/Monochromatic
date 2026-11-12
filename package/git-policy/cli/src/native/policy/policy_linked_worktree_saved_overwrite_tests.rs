//! What:
//!  Whether the worktree holds a directory at one path a form writes,
//!  which decides whether that entry is copied or moved whole.
//! Why:
//!  A symbolic link is copied as a link and a directory is moved whole;
//!  following a link would move the directory it names,
//!  which the command never touches,
//!  and calling a link a file would copy the only copy of a directory's contents
//!  one entry at a time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // isADirectory(worktree, 'nested') === true; isADirectory(worktree, 'link-to-nested') === false
//! ```

use super::is_a_directory;
use crate::command_test_support::{fixture, remove};

/// A directory is a directory,
/// a link to one is a link,
/// a file is neither,
/// and a path that is gone is nothing to move whole.
#[test]
fn a_directory_is_told_apart_from_a_link_and_a_file() {
    let root = fixture("discard-saved-overwrite-directory");
    std::fs::create_dir_all(root.join("nested/inner")).expect("the directory in the way");
    std::fs::write(root.join("b.txt"), b"untracked\n").expect("the file in the way");
    std::os::unix::fs::symlink(root.join("nested"), root.join("link")).expect("the link in the way");
    assert!(is_a_directory(root.as_path(), "nested"));
    assert!(!is_a_directory(root.as_path(), "link"));
    assert!(!is_a_directory(root.as_path(), "b.txt"));
    assert!(!is_a_directory(root.as_path(), "gone"));
    remove(root.as_path());
}
