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
/// with every entry at its paths written over,
/// and no guess of a remote branch unless `guesses` says `checkout` or `switch` makes one.
fn tree(revision: Option<&str>, guesses: bool) -> Option<WrittenPaths> {
    return Some(WrittenPaths::Tree {
        revision: revision.map(OsString::from),
        entries: WrittenEntries::EveryEntry,
        guesses_a_remote_branch: guesses,
    });
}

/// The whole tree of a named revision,
/// with only the entries the ignore rules cover written over;
/// a reset never guesses a remote branch.
fn tree_of_ignored_entries(revision: Option<&str>) -> Option<WrittenPaths> {
    return Some(WrittenPaths::Tree {
        revision: revision.map(OsString::from),
        entries: WrittenEntries::OnlyIgnored,
        guesses_a_remote_branch: false,
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
            written_paths(GuardedCommand::ReadTree, words(values.as_slice()).as_slice()),
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
        (vec!["--hard"], tree(None, false)),
        (vec!["--hard", "other"], tree(Some("other"), false)),
        (vec!["other", "--hard"], tree(Some("other"), false)),
        (vec!["--merge", "other"], tree_of_ignored_entries(Some("other"))),
        (vec!["--keep", "other"], tree_of_ignored_entries(Some("other"))),
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
        (vec!["--force"], tree(None, true)),
        (vec!["--force", "other"], tree(Some("other"), true)),
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
        (vec!["--force", "other"], tree(Some("other"), true)),
        (vec!["--discard-changes", "other"], tree(Some("other"), true)),
        (vec!["--force"], tree(None, true)),
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
        assert_eq!(written_paths(command, words(&["--force", "other"]).as_slice()), None, "{command:?}");
    }
}

/// What:
///  A forced `mv` names its sources and its destination,
///  and the worktree decides what Git writes from them
///  (a destination that is a directory is moved into, one that is not is renamed onto).
/// Why:
///  The reading cannot know which shape it has without the filesystem,
///  and a reading that guessed named the destination directory itself as written over,
///  which refused a safe `git mv -f a.txt b` (the third review's defect 1).
///  The shapes themselves are read in `repository_facts_written_paths_tests.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writtenPaths('Move', ['--force', 'a.txt', 'b']) === { kind: 'move-into', sources: ['a.txt'], destination: 'b' }
/// ```
#[test]
fn a_forced_move_names_its_sources_and_its_destination() {
    let cases: [(Vec<&str>, Vec<&str>, &str); 5] = [
        (vec!["--force", "a.txt", "b.txt"], vec!["a.txt"], "b.txt"),
        (vec!["-f", "a.txt", "dir/b.txt"], vec!["a.txt"], "dir/b.txt"),
        (vec!["--force", "a.txt", "dir"], vec!["a.txt"], "dir"),
        (vec!["--force", "a.txt", "x.txt", "dir"], vec!["a.txt", "x.txt"], "dir"),
        (vec!["--force", "a.txt", "--", "b.txt"], vec!["a.txt"], "b.txt"),
    ];
    for (values, wanted_sources, wanted_destination) in cases {
        assert_eq!(
            written_paths(GuardedCommand::Move, words(values.as_slice()).as_slice()),
            Some(WrittenPaths::MoveInto {
                sources: words(wanted_sources.as_slice()),
                destination: OsString::from(wanted_destination),
            }),
            "{values:?}"
        );
    }
    // A move with no source names nothing: Git refuses it.
    assert_eq!(written_paths(GuardedCommand::Move, words(&["--force", "b"]).as_slice()), None);
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
            written_paths(GuardedCommand::CheckoutIndex, words(values.as_slice()).as_slice()),
            expected,
            "{values:?}"
        );
    }
}

/// What:
///  The pathspec-file answer a form gives:
///  what the list is matched against,
///  the file,
///  its separator,
///  which entries an empty list turns the form into,
///  and whether `checkout` may guess the remote branch its name stands for.
/// Why:
///  Every spelling of the option reaches the same reading, and each field is a different question the
///  files in the way are found by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fromAFile('tree', 'list.txt', false, 'every-entry', true)
/// ```
fn from_a_file(
    target: PathspecTarget,
    file: &str,
    nul_separated: bool,
    entries: WrittenEntries,
    guesses: bool,
) -> Option<WrittenPaths> {
    return Some(WrittenPaths::PathspecFile {
        target,
        file: OsString::from(file),
        nul_separated,
        entries,
        guesses_a_remote_branch: guesses,
    });
}

/// A form that reads its pathspecs from a file names paths no token does,
/// in every spelling Git accepts,
/// matched against the tree it names or against the index when it names none;
/// a `restore` refuses an empty list itself, so its whole-tree answer keeps `EveryEntry`,
/// while a `checkout` writes over every entry when it is forced and over the ignored ones when it is not,
/// and guesses the remote branch its name stands for.
#[test]
fn a_pathspec_file_is_read_in_every_spelling() {
    let cases: [(GuardedCommand, Vec<&str>, Option<WrittenPaths>); 10] = [
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false, WrittenEntries::EveryEntry, false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false, WrittenEntries::EveryEntry, false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Index, "list.txt", false, WrittenEntries::EveryEntry, false),
        ),
        (
            GuardedCommand::RestoreWorktree,
            vec!["--source=other", "--pathspec-file-nul", "--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", true, WrittenEntries::EveryEntry, false),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false, WrittenEntries::OnlyIgnored, true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--pathspec-from-file=list.txt"],
            from_a_file(PathspecTarget::Index, "list.txt", false, WrittenEntries::OnlyIgnored, true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--pathspec-file-nul", "--pathspec-from-file=list.txt", "other"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", true, WrittenEntries::OnlyIgnored, true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--force", "other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false, WrittenEntries::EveryEntry, true),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--no-guess", "other", "--pathspec-from-file", "list.txt"],
            from_a_file(PathspecTarget::Tree(Some(OsString::from("other"))), "list.txt", false, WrittenEntries::OnlyIgnored, false),
        ),
        (
            GuardedCommand::CheckoutPaths,
            vec!["--force", "other"],
            tree(Some("other"), true),
        ),
    ];
    for (command, values, expected) in cases {
        assert_eq!(
            written_paths(command, words(values.as_slice()).as_slice()),
            expected,
            "{command:?} {values:?}"
        );
    }
}

//region The second review's readings of repeated options and of names that are not text

/// A source written twice is the last one, which is the value Git's own parser keeps.
#[test]
fn a_source_written_twice_is_the_last_one() {
    for (values, wanted) in [
        (vec!["-s", "main", "-s", "other", "--", "f.txt"], "other"),
        (vec!["--source=main", "--source=other", "--", "f.txt"], "other"),
        (vec!["--source", "main", "--source", "other", "--", "f.txt"], "other"),
        (vec!["-smain", "-sother", "--", "f.txt"], "other"),
        // A value that looks like a flag is that value, and the one after it is the last.
        (vec!["--source", "-W", "--source", "other", "--", "f.txt"], "other"),
    ] {
        let tree_of: Option<WrittenPaths> = matching(Some(wanted), &["f.txt"]);
        assert_eq!(
            written_paths(GuardedCommand::RestoreWorktree, words(values.as_slice()).as_slice()),
            tree_of,
            "{values:?}"
        );
    }
    for values in [vec!["-s", "other", "-s", "main", "--", "f.txt"], vec!["--source=other", "--source=main", "--", "f.txt"]] {
        assert_eq!(
            written_paths(GuardedCommand::RestoreWorktree, words(values.as_slice()).as_slice()),
            matching(Some("main"), &["f.txt"]),
            "{values:?}"
        );
    }
}

/// Every word that is a path or a revision keeps the bytes it was typed as,
/// in each spelling Git's tables accept.
#[test]
fn a_word_that_is_a_path_or_a_revision_keeps_its_bytes() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    /// One name as the operating system holds it, with a byte no text holds.
    fn word(bytes: &[u8]) -> OsString {
        return OsString::from_vec(bytes.to_vec());
    }
    // A `read-tree` prefix, and a `checkout-index` prefix.
    let read_tree_prefix: [u8; 9] = [b'p', b'-', 0xff, b'/', b'x', b'/', b'y', b'/', b'z'];
    let read_tree: Option<WrittenPaths> = written_paths(
        GuardedCommand::ReadTree,
        &[word(b"-u"), word(b"--prefix"), word(&read_tree_prefix), word(b"other")],
    );
    assert_eq!(
        read_tree,
        Some(WrittenPaths::ReadTree {
            revisions: vec![word(b"other")],
            prefix: Some(word(&read_tree_prefix)),
            entries: WrittenEntries::OnlyIgnored,
        })
    );
    let checkout_index: Option<WrittenPaths> = written_paths(
        GuardedCommand::CheckoutIndex,
        &[word(b"-f"), word(b"--prefix=p-\xff/"), word(b"-a")],
    );
    assert_eq!(
        checkout_index,
        Some(WrittenPaths::IndexUnderPrefix {
            pathspecs: Vec::<OsString>::new(),
            prefix: word(b"p-\xff/"),
        })
    );
    // Both spellings of a restore's source.
    for region in [
        vec![word(b"--source=br-\xff"), word(b"--"), word(b"f.txt")],
        vec![word(b"-sbr-\xff"), word(b"--"), word(b"f.txt")],
    ] {
        assert_eq!(
            written_paths(GuardedCommand::RestoreWorktree, region.as_slice()),
            Some(WrittenPaths::MatchingTree {
                revision: Some(word(b"br-\xff")),
                pathspecs: vec![word(b"f.txt")],
            })
        );
    }
    // The file a `--pathspec-from-file` name gives.
    let inline: Option<WrittenPaths> = written_paths(
        GuardedCommand::CheckoutPaths,
        &[word(b"other"), word(b"--pathspec-from-file=n-\xff")],
    );
    match inline {
        Some(WrittenPaths::PathspecFile { file, .. }) => assert_eq!(file, word(b"n-\xff")),
        other => panic!("the inline file name was read as {other:?}"),
    }
    // A destination of a forced move that is not text keeps its bytes.
    let moved: Option<WrittenPaths> = written_paths(
        GuardedCommand::Move,
        &[word(b"-f"), word(b"a.txt"), word(b"p-\xff")],
    );
    match moved {
        Some(WrittenPaths::MoveInto { destination, .. }) => assert_eq!(destination, word(b"p-\xff")),
        other => panic!("the move was read as {other:?}"),
    }
}

//endregion

//region The fourth review: a word the reading mistook (2026-11-14)

/// What:
///  A revision spelled exactly like the file `--pathspec-from-file` names is still the revision.
/// Why:
///  The reading used to drop every positional word equal to the file's name,
///  which was there to skip a separated value that the option's own table already consumes.
///  `git checkout -f topic --pathspec-from-file=topic` then named no revision,
///  so the form was read as aimed at the index,
///  whose paths do not hold the tree's new `b.txt`,
///  and the untracked file at that path was written over with no refusal
///  (measured on this host with native Git 2.56.0,
///  2026-11-14).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writtenPaths('CheckoutPaths', ['-f', 'topic', '--pathspec-from-file=topic']).target === { kind: 'tree', revision: 'topic' }
/// ```
#[test]
fn a_pathspec_file_named_like_the_revision_is_still_the_revision() {
    assert_eq!(
        written_paths(
            GuardedCommand::CheckoutPaths,
            words(&["-f", "topic", "--pathspec-from-file=topic"]).as_slice()
        ),
        from_a_file(PathspecTarget::Tree(Some(OsString::from("topic"))), "topic", false, WrittenEntries::EveryEntry, true)
    );
    // The separated spelling puts the file's name with its option, so the revision is the only positional word left.
    assert_eq!(
        written_paths(
            GuardedCommand::CheckoutPaths,
            words(&["-f", "--pathspec-from-file", "list", "topic"]).as_slice()
        ),
        from_a_file(PathspecTarget::Tree(Some(OsString::from("topic"))), "list", false, WrittenEntries::EveryEntry, true)
    );
    // A form naming no revision still writes the index's paths.
    assert_eq!(
        written_paths(
            GuardedCommand::CheckoutPaths,
            words(&["-f", "--pathspec-from-file=topic"]).as_slice()
        ),
        from_a_file(PathspecTarget::Index, "topic", false, WrittenEntries::EveryEntry, true)
    );
}

/// What:
///  A `--prefix` value written as the next token is that option's value and no pathspec.
/// Why:
///  The reading consumed the value but did not step past it,
///  so the value was read a second time as a path:
///  `git checkout-index -f --prefix out/ -- a.txt` wrote the index paths matching `out/` below
///  `out/` again, naming `out/out/keep.txt`,
///  a path Git never writes,
///  and the wrapper refused a command real Git runs
///  (measured on this host with native Git 2.56.0,
///  2026-11-14).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writtenPaths('CheckoutIndex', ['-f', '--prefix', 'out/', '--', 'a.txt']) === { kind: 'index-under-a-prefix', pathspecs: ['a.txt'], prefix: 'out/' }
/// ```
#[test]
fn a_separated_checkout_index_prefix_is_no_pathspec() {
    assert_eq!(
        written_paths(
            GuardedCommand::CheckoutIndex,
            words(&["-f", "--prefix", "out/", "--", "a.txt"]).as_slice()
        ),
        Some(WrittenPaths::IndexUnderPrefix {
            pathspecs: words(&["a.txt"]),
            prefix: OsString::from("out/"),
        })
    );
    // A pathspec that really names an index path below the prefix is still read, and still shifted.
    assert_eq!(
        written_paths(
            GuardedCommand::CheckoutIndex,
            words(&["-f", "--prefix", "out/", "--", "out/keep.txt"]).as_slice()
        ),
        Some(WrittenPaths::IndexUnderPrefix {
            pathspecs: words(&["out/keep.txt"]),
            prefix: OsString::from("out/"),
        })
    );
}

//endregion
