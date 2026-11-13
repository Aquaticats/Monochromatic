//! What:
//!  The forms whose written paths are read,
//!  the forms that write none,
//!  and every spelling a path form reaches through.
//! Why:
//!  A refusal is the guard's answer to an untracked file in the way,
//!  and the entries it names come from this reading:
//!  a spelling read here as "the whole tree" after a form Git would write only one path for
//!  would refuse a command over files it never touches,
//!  and a spelling read as "nothing" is a file lost with no refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // writtenPaths('RestoreWorktree', ['--source=other', '--', 'b.txt'])
//! //   === { kind: 'matching-tree', revision: 'other', pathspecs: ['b.txt'] }
//! ```

use super::{PathspecTarget, WrittenEntries, WrittenPaths, written_paths};
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;

/// The words as arguments, each its own.
fn words(values: &[&str]) -> Vec<OsString> {
    return values.iter().map(OsString::from).collect();
}

/// The whole tree of a named revision,
/// with every entry at its paths written over.
fn tree(revision: Option<&str>) -> Option<WrittenPaths> {
    return Some(WrittenPaths::Tree {
        revision: revision.map(OsString::from),
        entries: WrittenEntries::EveryEntry,
    });
}

/// The whole tree of a named revision,
/// with only the entries the ignore rules cover written over.
fn tree_of_ignored_entries(revision: Option<&str>) -> Option<WrittenPaths> {
    return Some(WrittenPaths::Tree {
        revision: revision.map(OsString::from),
        entries: WrittenEntries::OnlyIgnored,
    });
}

/// The paths one revision's tree holds that these pathspecs reach.
fn matching(revision: Option<&str>, pathspecs: &[&str]) -> Option<WrittenPaths> {
    return Some(WrittenPaths::MatchingTree {
        revision: revision.map(OsString::from),
        pathspecs: words(pathspecs),
    });
}

/// The paths of every tree a `git read-tree` form names,
/// below the directory it writes them under,
/// and which entries at them it writes over.
fn read_tree(revisions: &[&str], prefix: Option<&str>, entries: WrittenEntries) -> Option<WrittenPaths> {
    return Some(WrittenPaths::ReadTree {
        revisions: words(revisions),
        prefix: prefix.map(OsString::from),
        entries,
    });
}

/// A `read-tree` writes the paths of every tree it names,
/// below the directory `--prefix` puts them in,
/// and writes over the untracked entries too only in the reset form:
/// measured on this host with native Git 2.56.0,
/// `--reset -u` replaces an untracked file at such a path and exits 0,
/// while `-m -u` refuses one and writes over the ignored one.
#[test]
fn a_read_tree_writes_the_trees_it_names_below_its_prefix() {
    for (values, expected) in [
        (vec!["-m", "-u", "HEAD", "other"], read_tree(&["HEAD", "other"], None, WrittenEntries::OnlyIgnored)),
        (vec!["-m", "-u", "other"], read_tree(&["other"], None, WrittenEntries::OnlyIgnored)),
        (vec!["--reset", "-u", "other"], read_tree(&["other"], None, WrittenEntries::EveryEntry)),
        (vec!["--res", "-u", "other"], read_tree(&["other"], None, WrittenEntries::EveryEntry)),
        (vec!["--prefix=sub/", "-u", "other"], read_tree(&["other"], Some("sub/"), WrittenEntries::OnlyIgnored)),
        (vec!["--prefix", "sub/", "-u", "other"], read_tree(&["other"], Some("sub/"), WrittenEntries::OnlyIgnored)),
        // No `-u` writes no worktree file, and a form Git refuses writes none either.
        (vec!["-m", "HEAD"], None),
        (vec!["--reset", "HEAD"], None),
        (vec!["-u", "other"], None),
        // A dry run and `--debug-unpack` return before anything is written.
        (vec!["-n", "-m", "-u", "other"], None),
        (vec!["--dry-run", "--reset", "-u", "other"], None),
        (vec!["--debug-unpack", "-m", "-u", "other"], None),
        // Git refuses a form with no tree to read.
        (vec!["-u", "--reset"], None),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::ReadTree, words(values.as_slice()).as_slice(), None),
            expected,
            "{values:?}"
        );
    }
}

/// A hard reset writes the named tree whole,
/// a merge or keep reset writes the tree over the ignored entries in its way alone,
/// because Git keeps an untracked one and refuses itself,
/// and a soft or mixed one is not read at all.
#[test]
fn a_reset_writes_the_tree_it_names() {
    for (values, expected) in [
        (vec!["--hard"], tree(None)),
        (vec!["--hard", "other"], tree(Some("other"))),
        (vec!["other", "--hard"], tree(Some("other"))),
        (vec!["--merge", "other"], tree_of_ignored_entries(Some("other"))),
        (vec!["--keep", "other"], tree_of_ignored_entries(Some("other"))),
        (vec!["--soft", "other"], None),
        (vec!["other"], None),
    ] {
        assert_eq!(
            written_paths(GuardedCommand::ResetHard, words(values.as_slice()).as_slice(), None),
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
            written_paths(GuardedCommand::CheckoutPaths, words(values.as_slice()).as_slice(), None),
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
            written_paths(GuardedCommand::SwitchForce, words(values.as_slice()).as_slice(), None),
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
            written_paths(GuardedCommand::RestoreWorktree, words(values.as_slice()).as_slice(), None),
            expected,
            "{values:?}"
        );
    }
}

/// Every other guarded form asks no written-path question here:
/// a clean deletes what its own dry run names,
/// and a `rm` and a `stash` reach only paths their index already holds.
#[test]
fn the_other_forms_write_no_tree_path() {
    for command in [
        GuardedCommand::Clean,
        GuardedCommand::CleanIgnored,
        GuardedCommand::CleanIgnoredOnly,
        GuardedCommand::ResetMixed,
        GuardedCommand::Remove,
        GuardedCommand::Stash,
    ] {
        assert_eq!(written_paths(command, words(&["--force", "other"]).as_slice(), None), None, "{command:?}");
    }
}

/// A forced `mv` writes its destination and the name each source takes inside a directory destination,
/// which are paths no tree or index query names.
#[test]
fn a_forced_move_writes_its_destination() {
    // The reading cannot know whether the destination is a directory without asking the filesystem,
    //  so it names both the destination and the name a source would take inside it;
    //  a path the worktree does not hold is dropped before anything is refused.
    let cases: [(Vec<&str>, Vec<&str>); 4] = [
        (vec!["--force", "a.txt", "b.txt"], vec!["b.txt", "b.txt/a.txt"]),
        (vec!["-f", "a.txt", "dir/b.txt"], vec!["dir/b.txt", "dir/b.txt/a.txt"]),
        (vec!["--force", "a.txt", "dir"], vec!["dir", "dir/a.txt"]),
        (vec!["--force", "a.txt", "--", "b.txt"], vec!["b.txt", "b.txt/a.txt"]),
    ];
    for (values, expected) in cases {
        let answer: Option<WrittenPaths> = written_paths(GuardedCommand::Move, words(values.as_slice()).as_slice(), None);
        match answer {
            Some(WrittenPaths::Paths { paths }) => {
                let mut shown: Vec<String> = paths.iter().map(|path| return path.to_string_lossy().into_owned()).collect();
                shown.sort();
                let mut wanted: Vec<String> = expected.iter().map(|path| return String::from(*path)).collect();
                wanted.sort();
                assert_eq!(shown, wanted, "{values:?}");
            }
            other => panic!("{values:?} read as {other:?}"),
        }
    }
}

/// A forced `checkout-index` writes the index's paths, all of them with `-a`,
/// and each below the directory `--prefix` names.
#[test]
fn a_forced_checkout_index_writes_the_index_below_its_prefix() {
    let cases: [(Vec<&str>, Option<WrittenPaths>); 5] = [
        (vec!["--force", "-a"], Some(WrittenPaths::Index { pathspecs: Vec::new() })),
        (vec!["-f", "--all"], Some(WrittenPaths::Index { pathspecs: Vec::new() })),
        (vec!["--force", "a.txt"], Some(WrittenPaths::Index { pathspecs: words(&["a.txt"]) })),
        (
            vec!["--force", "--prefix=out/", "-a"],
            Some(WrittenPaths::IndexUnderPrefix {
                pathspecs: Vec::new(),
                prefix: OsString::from("out/"),
            }),
        ),
        (
            vec!["--force", "--prefix", "out/", "-a"],
            Some(WrittenPaths::IndexUnderPrefix {
                pathspecs: Vec::new(),
                prefix: OsString::from("out/"),
            }),
        ),
    ];
    for (values, expected) in cases {
        assert_eq!(
            written_paths(GuardedCommand::CheckoutIndex, words(values.as_slice()).as_slice(), None),
            expected,
            "{values:?}"
        );
    }
}

/// A form that reads its pathspecs from a file names paths no token does,
/// in every spelling Git accepts,
/// matched against the tree it names or against the index when it names none.
#[test]
fn a_pathspec_file_is_read_in_every_spelling() {
    let from_a_file: fn(PathspecTarget, &str, bool) -> Option<WrittenPaths> = |target, file, nul_separated| {
        return Some(WrittenPaths::PathspecFile {
            target,
            file: OsString::from(file),
            nul_separated,
            directory: None,
        });
    };
    let cases: [(GuardedCommand, Vec<&str>, Option<WrittenPaths>); 8] = [
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Index, "list.txt", false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-file-nul", "--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Index, "list.txt", false),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--pathspec-file-nul", "--pathspec-from-file=list.txt", "other"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--force", "other"],
            tree(Some("other")),
        ),
    ];
    for (command, values, expected) in cases {
        assert_eq!(
            written_paths(command, words(values.as_slice()).as_slice(), None),
            expected,
            "{command:?} {values:?}"
        );
    }
}
