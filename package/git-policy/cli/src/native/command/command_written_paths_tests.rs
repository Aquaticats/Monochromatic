//! What:
//!  The forms whose written paths are read,
//!  the forms that write none,
//!  and every spelling a path form reaches through.
//! Why:
//!  A saved copy is one part of the guard's answer to an untracked file in the way,
//!  and the paths it saves come from this reading:
//!  a spelling read here as "the whole tree" after a form Git would write only one path for
//!  would save files the command never touches,
//!  and a spelling read as "nothing" is a file lost with no copy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('RestoreWorktree', ['--source=other', '--', 'b.txt'])
//! //   === { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

use super::{WrittenPaths, written_paths};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;

/// The words as arguments, each its own.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// The whole tree of a named revision.
fn tree(revision: Option<&str>) -> Option<WrittenPaths> {
    return Some(WrittenPaths::Tree {
        revision: revision.map(OsString::from),
    });
}

/// The paths one revision's tree holds that these pathspecs reach.
fn matching(revision: Option<&str>, pathspecs: &[&str]) -> Option<WrittenPaths> {
    return Some(WrittenPaths::MatchingTree {
        revision: revision.map(OsString::from),
        pathspecs: words(pathspecs),
    });
}

/// A hard reset writes the named tree whole,
/// a merge or keep reset writes nothing Git would not refuse over,
/// and a soft one is not guarded at all.
#[test]
fn a_hard_reset_writes_the_tree_it_names() {
    for (values, expected) in [
        (vec!["--hard"], tree(None)),
        (vec!["--hard", "other"], tree(Some("other"))),
        (vec!["other", "--hard"], tree(Some("other"))),
        (vec!["--merge", "other"], None),
        (vec!["--keep", "other"], None),
        (vec!["--soft", "other"], None),
        (vec!["other"], None),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::ResetHard, words(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// A forced checkout with no path writes the whole tree,
/// and every spelling that names paths writes only what it names.
#[test]
fn a_checkout_writes_what_its_own_words_name() {
    for (values, expected) in [
        (vec!["--force"], tree(None)),
        (vec!["--force", "other"], tree(Some("other"))),
        (vec!["other", "--", "b.txt"], matching(Some("other"), &["b.txt"])),
        (vec!["--", "b.txt"], matching(None, &["b.txt"])),
        (vec!["other", "--", "--force"], matching(Some("other"), &["--force"])),
        (vec!["--force", "other", "--", "."], matching(Some("other"), &["."])),
        (vec!["other", "b.txt"], matching(Some("other"), &["b.txt"])),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::CheckoutPaths, words(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// A forced switch writes the whole tree it moves to,
/// and one that names no target writes `HEAD`'s.
#[test]
fn a_forced_switch_writes_the_whole_tree() {
    for (values, expected) in [
        (vec!["--force", "other"], tree(Some("other"))),
        (vec!["--discard-changes", "other"], tree(Some("other"))),
        (vec!["--force"], tree(None)),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::SwitchForce, words(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// A restore of the worktree writes the source tree's paths when it names one,
/// the index's paths when it does not,
/// and nothing at all when it restores only the index.
#[test]
fn a_restore_writes_the_paths_it_names_over() {
    for (values, expected) in [
        (vec!["--source=other", "--", "b.txt"], matching(Some("other"), &["b.txt"])),
        (vec!["-s", "other", "--", "b.txt"], matching(Some("other"), &["b.txt"])),
        (vec!["-sother", "--", "b.txt"], matching(Some("other"), &["b.txt"])),
        (vec!["--sour", "other", "b.txt"], matching(Some("other"), &["b.txt"])),
        (vec!["--source", "other", "--worktree", "."], matching(Some("other"), &["."])),
        (vec!["b.txt"], Some(WrittenPaths::Index { pathspecs: words(&["b.txt"]) })),
        (vec!["."], Some(WrittenPaths::Index { pathspecs: words(&["."]) })),
        (vec!["--staged", "--", "b.txt"], None),
        (vec!["--st", "--", "b.txt"], None),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::RestoreWorktree, words(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// Every other guarded form writes no tree path this save keeps,
/// because Git itself refuses over a file it did not put there.
#[test]
fn the_other_forms_write_no_tree_path() {
    for command in [
        GuardedCommand::Clean,
        GuardedCommand::CleanIgnored,
        GuardedCommand::CleanIgnoredOnly,
        GuardedCommand::ResetMixed,
        GuardedCommand::Remove,
        GuardedCommand::Move,
        GuardedCommand::CheckoutIndex,
        GuardedCommand::ReadTree,
        GuardedCommand::Stash,
    ] {
        assert_eq!(written_paths(command, words(&["--force", "other"]).as_slice()), None, "{command:?}");
    }
}
