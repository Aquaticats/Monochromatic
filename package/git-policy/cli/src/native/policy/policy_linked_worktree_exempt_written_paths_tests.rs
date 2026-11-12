//! What:
//!  The paths each guarded form's own words say it writes.
//! Why:
//!  A refusal that looks at the wrong paths either refuses a command Git would not harm
//!  or lets one through that deletes a file existing nowhere else,
//!  so each spelling that means a different set is decided here:
//!  a whole tree,
//!  the paths after `--`,
//!  the index's content,
//!  and a move's destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('CheckoutPaths', ['other', '--', 'b.txt'], root) === { kind: 'tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

use super::{Written, written_paths};
use crate::command_test_support::{fixture, os_arguments, remove};
use crate::rule_linked_worktree::GuardedCommand;
use std::path::PathBuf;

/// What:
///  The region of one form as a `Vec<OsString>`.
/// Why:
///  Every control is written as the person would type it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const region = words(['reset', '--hard', 'other']);
/// ```
fn words(region: &[&str]) -> Vec<std::ffi::OsString> {
    return os_arguments(region);
}

/// What:
///  The paths one form writes,
///  read in a directory that holds no destination,
///  which is the whole-tree answer of every form.
/// Why:
///  A move is the one form whose answer depends on the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const paths = read('ResetHard', ['--hard', 'other']);
/// ```
fn read(command: GuardedCommand, region: &[&str]) -> Option<Written> {
    return written_paths(command, words(region).as_slice(), PathBuf::from("/nonexistent").as_path())
        .expect("the form's words are read");
}

/// A hard reset writes the whole tree of the revision it names,
///  or `HEAD`'s,
/// and `--merge` and `--keep` write no untracked entry because Git refuses to replace one.
#[test]
fn a_hard_reset_writes_the_tree_it_names() {
    assert_eq!(
        read(GuardedCommand::ResetHard, &["--hard", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::ResetHard, &["--hard"]),
        Some(Written::Tree {
            revision: None,
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
    assert_eq!(read(GuardedCommand::ResetHard, &["--merge", "other"]), None);
    assert_eq!(read(GuardedCommand::ResetHard, &["--keep"]), None);
}

/// A forced checkout with no path writes the whole tree,
/// and a path form writes exactly the paths it names,
/// from the revision before `--` or from the index when it names none.
#[test]
fn a_checkout_writes_its_whole_tree_or_the_paths_it_names() {
    assert_eq!(
        read(GuardedCommand::CheckoutPaths, &["--force", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::CheckoutPaths, &["other", "--", "inway.txt"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::CheckoutPaths, &["--", "inway.txt"]),
        Some(Written::Index {
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::CheckoutPaths, &["other", "inway.txt"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
}

/// A forced switch writes the whole tree of the branch or commit it moves to.
#[test]
fn a_forced_switch_writes_the_tree_it_moves_to() {
    assert_eq!(
        read(GuardedCommand::SwitchForce, &["--force", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
}

/// A restore writes its source tree's paths,
///  the index's when it names no source,
/// and no worktree path at all when it restores the index alone.
#[test]
fn a_restore_writes_its_source_or_the_index() {
    assert_eq!(
        read(GuardedCommand::RestoreWorktree, &["--source=other", "--", "inway.txt"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::RestoreWorktree, &["--source", "other", "--", "inway.txt"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::RestoreWorktree, &["-s", "other", "--", "inway.txt"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::RestoreWorktree, &["--", "inway.txt"]),
        Some(Written::Index {
            pathspecs: os_arguments(&["inway.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(read(GuardedCommand::RestoreWorktree, &["--staged", "--", "inway.txt"]), None);
}

/// A `read-tree` writes the tree it is given last,
///  below its prefix,
/// and names no tree at all when it empties the index.
#[test]
fn a_read_tree_writes_the_tree_it_is_given_last() {
    assert_eq!(
        read(GuardedCommand::ReadTree, &["--reset", "-u", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::ReadTree, &["-m", "-u", "HEAD", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: Vec::new(),
        })
    );
    assert_eq!(
        read(GuardedCommand::ReadTree, &["--reset", "-u", "--prefix=sub/", "other"]),
        Some(Written::Tree {
            revision: Some(std::ffi::OsString::from("other")),
            pathspecs: Vec::new(),
            prefix: b"sub/".to_vec(),
        })
    );
    assert_eq!(read(GuardedCommand::ReadTree, &["--empty", "-u"]), None);
    assert_eq!(read(GuardedCommand::ReadTree, &["--reset", "other"]), None);
}

/// What:
///  A move writes its destination,
///  and one into an existing directory writes that directory joined with each source's name.
/// Why:
///   The two spellings replace different files,
///  and only the worktree says which one Git writes.
#[test]
fn a_move_writes_its_destination() {
    assert_eq!(
        read(GuardedCommand::Move, &["--force", "src.txt", "dst.txt"]),
        Some(Written::Paths {
            paths: vec![b"dst.txt".to_vec()],
        })
    );
    let root: PathBuf = fixture("written-paths-move-directory");
    std::fs::create_dir_all(root.join("into")).expect("the destination directory");
    let written: Option<Written> = written_paths(
        GuardedCommand::Move,
        os_arguments(&["--force", "src.txt", "into"]).as_slice(),
        root.as_path(),
    )
    .expect("the move's words are read");
    assert_eq!(
        written,
        Some(Written::Paths {
            paths: vec![b"into/src.txt".to_vec()],
        })
    );
    remove(root.as_path());
}

/// A `checkout-index` writes the index's paths it names,
///  or every one of them with `--all`,
///  each under its prefix.
#[test]
fn a_checkout_index_writes_the_index_below_its_prefix() {
    assert_eq!(
        read(GuardedCommand::CheckoutIndex, &["--force", "--prefix=sub/", "--all"]),
        Some(Written::Index {
            pathspecs: Vec::new(),
            prefix: b"sub/".to_vec(),
        })
    );
    assert_eq!(
        read(GuardedCommand::CheckoutIndex, &["--force", "--", "file.txt"]),
        Some(Written::Index {
            pathspecs: os_arguments(&["file.txt"]),
            prefix: Vec::new(),
        })
    );
    assert_eq!(read(GuardedCommand::CheckoutIndex, &["--force"]), None);
}

/// A form that writes no worktree file from a tree or the index has no untracked entry to refuse over.
#[test]
fn a_form_that_writes_no_worktree_file_names_no_paths() {
    assert_eq!(read(GuardedCommand::Clean, &["-fd"]), None);
    assert_eq!(read(GuardedCommand::Stash, &["push"]), None);
    assert_eq!(read(GuardedCommand::Remove, &["-f", "tracked.txt"]), None);
}
