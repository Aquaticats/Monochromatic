//! Controls for the readers of the options that stop or shape a clean run, the cutting of the conflict hint, and the default
//!  of the editor. The finish as a whole runs through the built executable in `binary_word_merge_finish_tests.rs`.

use super::*;
use std::path::{Path, PathBuf};

/// Command-line words as operating-system text.
fn words(text: &[&str]) -> Vec<OsString> {
    return text.iter().map(OsString::from).collect();
}

/// The stops of a merge, or nothing for a command line that is not read.
fn merge_of(text: &[&str]) -> Option<Stops> {
    return merge_stops(words(text).as_slice(), false);
}

/// Each option that stops or shapes a clean merge is read by its exact spelling, and the last of two opposites wins.
#[test]
fn merge_options_are_read_by_exact_spelling_and_the_last_opposite_wins() {
    assert_eq!(merge_of(&["--no-commit", "topic"]), Some(Stops { no_commit: true, ..Stops::default() }));
    assert_eq!(merge_of(&["--no-commit", "--commit", "topic"]), Some(Stops::default()));
    assert_eq!(merge_of(&["--squash", "topic"]), Some(Stops { squash: true, ..Stops::default() }));
    assert_eq!(merge_of(&["--squash", "--no-squash", "topic"]), Some(Stops::default()));
    assert_eq!(merge_of(&["-e", "topic"]).and_then(|stops| { return stops.edit; }), Some(true));
    assert_eq!(merge_of(&["--edit", "--no-edit", "topic"]).and_then(|stops| { return stops.edit; }), Some(false));
    let named: Stops = merge_of(&["-m", "text", "--file=notes", "topic"]).expect("read");
    assert!(named.message);
    assert_eq!(merge_of(&["-S", "--signoff", "topic"]).expect("read").passed_on, words(&["-S", "--signoff"]));
    assert_eq!(merge_of(&["-s", "ours", "--strategy=ort", "-Xpatience", "topic"]), None, "-Xpatience is attached, which is not read");
    assert!(merge_of(&["-s", "ours", "--strategy=ort", "--ff-only", "topic"]).is_some());
}

/// An option this does not know, which may be an abbreviation of one that stops, leaves the operation to the person.
#[test]
fn an_unknown_option_or_an_abbreviation_is_not_read() {
    assert_eq!(merge_of(&["--no-comm", "topic"]), None);
    assert_eq!(merge_of(&["--squa", "topic"]), None);
    assert_eq!(merge_of(&["-m"]), None, "a value is missing");
    assert_eq!(pick_stops(words(&["--no-comm", "x"]).as_slice()), None);
}

/// Pull reads the fetch and rebase options that merge does not, and only for pull.
#[test]
fn pull_options_are_read_only_for_pull() {
    assert!(merge_stops(words(&["--rebase", "--prune", "origin"]).as_slice(), true).is_some());
    assert!(merge_stops(words(&["--rebase", "origin"]).as_slice(), false).is_none());
}

/// A cherry-pick or revert reads `-n`, the edit options and a continue form, and signing options are handed on.
#[test]
fn pick_options_are_read() {
    let (without_commit, without_continuing) = pick_stops(words(&["-n", "-x", "-m", "1", "topic"]).as_slice()).expect("read");
    assert!(without_commit.no_commit && !without_continuing);
    let (resumed, resuming) = pick_stops(words(&["--continue"]).as_slice()).expect("read");
    assert!(resuming && !resumed.no_commit);
    let (shaped, _) = pick_stops(words(&["--no-edit", "-s", "-Skey", "a..b"]).as_slice()).expect("read");
    assert_eq!(shaped.edit, Some(false));
    assert_eq!(shaped.passed_on, words(&["--signoff", "-Skey"]));
}

/// The options Git saved for a sequence are read from `sequencer/opts`; a missing file is nothing.
#[test]
fn the_saved_options_of_a_sequence_are_read() {
    let root: PathBuf = crate::test_support::fixture("finish-saved-opts");
    std::fs::create_dir_all(root.join("sequencer")).expect("directory");
    assert_eq!(saved_pick_stops(root.as_path()), None);
    std::fs::write(root.join("sequencer/opts"), "[options]\n\tno-commit = true\n\tedit = false\n\tsignoff = true\n").expect("the file");
    let saved: Stops = saved_pick_stops(root.as_path()).expect("read");
    assert!(saved.no_commit);
    assert_eq!(saved.edit, Some(false));
    assert_eq!(saved.passed_on, words(&["--signoff"]));
    crate::test_support::remove(root.as_path());
}

/// The hint Git appends is cut with its blank line, for the usual comment character and another; a message without it stays.
#[test]
fn the_conflict_hint_is_cut_and_nothing_else() {
    let hinted: &[u8] = b"Merge branch 'topic'\n\n# Conflicts:\n#\ta.md\n#\tdir/b.md\n";
    assert_eq!(without_conflicts_hint(hinted), b"Merge branch 'topic'\n".to_vec());
    let other: &[u8] = b"Subject\n\nBody line\n\n; Conflicts:\n;\ta.md\n";
    assert_eq!(without_conflicts_hint(other), b"Subject\n\nBody line\n".to_vec());
    for plain in [b"Merge branch 'topic'\n".as_slice(), b"Subject\n\nConflicts: none\n", b"Subject\n\n# a comment line\n", b""] {
        assert_eq!(without_conflicts_hint(plain), plain.to_vec());
    }
    let between: &[u8] = b"Subject\n\n# Conflicts:\n# prose that is not a path line\n";
    assert_eq!(without_conflicts_hint(between), between.to_vec(), "a line after `Conflicts:` that is not a path line is no hint");
}

/// A merge told a message does not open the editor; the variable decides next; a value Git would refuse is nothing.
#[test]
fn the_default_of_the_merge_editor_follows_git() {
    let none: Vec<(OsString, OsString)> = Vec::new();
    assert_eq!(merge_edits_by_default(true, none.as_slice()), Some(false));
    let environment = |value: &str| -> Vec<(OsString, OsString)> { return vec![(OsString::from("GIT_MERGE_AUTOEDIT"), OsString::from(value))]; };
    assert_eq!(merge_edits_by_default(false, environment("no").as_slice()), Some(false));
    assert_eq!(merge_edits_by_default(false, environment("yes").as_slice()), Some(true));
    assert_eq!(merge_edits_by_default(false, environment("maybe").as_slice()), None);
}

/// One more finish is nested while fewer than the budget run, and not at the budget.
#[test]
fn nesting_stops_at_the_budget() {
    assert!(nesting_allows(0, 1));
    assert!(!nesting_allows(1, 1), "a merge has one finish");
    assert!(nesting_allows(2, 3));
    assert!(!nesting_allows(3, 3));
}

/// A rebase todo list asks for an editor at `reword`, `squash` and `fixup -c`/`-C`, and not at the plain commands.
#[test]
fn a_todo_list_asks_for_an_editor_only_where_a_message_is_edited() {
    let root: PathBuf = crate::test_support::fixture("finish-todo");
    std::fs::create_dir_all(root.join("rebase-merge")).expect("directory");
    let todo = |text: &str| -> bool {
        std::fs::write(root.join("rebase-merge/git-rebase-todo"), text).expect("the todo list");
        return todo_needs_editor(root.as_path());
    };
    assert!(!todo("pick 1a2b3c subject\npick 4d5e6f subject\nfixup 7a8b9c subject\nexec true\n# a comment about reword\n"));
    assert!(todo("pick 1a2b3c subject\nreword 4d5e6f subject\n"));
    assert!(todo("pick 1a2b3c subject\nsquash 4d5e6f subject\n"));
    assert!(todo("pick 1a2b3c subject\nfixup -C 4d5e6f subject\n"));
    assert!(todo("r 4d5e6f subject\n"));
    std::fs::remove_file(root.join("rebase-merge/git-rebase-todo")).expect("remove");
    assert!(!todo_needs_editor(root.as_path()), "no todo list, no editor");
    crate::test_support::remove(root.as_path());
}

/// A todo file with `lines` in it, written under `directory`.
fn todo_file(root: &Path, directory: &str, name: &str, lines: &[&str]) {
    std::fs::create_dir_all(root.join(directory)).expect("directory");
    std::fs::write(root.join(directory).join(name), lines.join("\n") + "\n").expect("the todo list");
}

/// The number of finishes one outermost finish may nest comes from the input: a rebase's remaining todo length plus one, a pick's
///  sequencer todo length (one for a single pick), one for a merge; comments and blank lines are not commands.
#[test]
fn the_nesting_bound_is_derived_from_the_todo_list() {
    let root: PathBuf = crate::test_support::fixture("finish-budget");
    assert_eq!(finish_budget(Operation::Merge, root.as_path()), 1);
    assert_eq!(finish_budget(Operation::Rebase, root.as_path()), 1, "no remaining todo: the finish itself");
    assert_eq!(finish_budget(Operation::Pick, root.as_path()), 1, "a single pick has no sequencer todo");
    todo_file(root.as_path(), "rebase-merge", "git-rebase-todo", &["pick 1a subject", "# a comment", "", "pick 2b subject", "exec true"]);
    assert_eq!(finish_budget(Operation::Rebase, root.as_path()), 4, "three commands plus one");
    todo_file(root.as_path(), "sequencer", "todo", &["pick 1a subject", "pick 2b subject", "# comment"]);
    assert_eq!(finish_budget(Operation::Pick, root.as_path()), 2);
    assert_eq!(finish_budget(Operation::Merge, root.as_path()), 1, "a merge is one finish whatever else is there");
    crate::test_support::remove(root.as_path());
}

/// `branch.<name>.mergeoptions` is split as Git splits it: on white space, with quotes and backslashes (`split_cmdline`).
#[test]
fn a_merge_options_value_is_split_as_git_splits_it() {
    assert_eq!(split_cmdline("--no-edit -m \"a message with spaces\""), Some(words(&["--no-edit", "-m", "a message with spaces"])));
    assert_eq!(split_cmdline("  --squash   --edit "), Some(words(&["--squash", "--edit"])));
    assert_eq!(split_cmdline("-m 'single \\ quoted' two\\ words"), Some(words(&["-m", "single \\ quoted", "two words"])));
    assert_eq!(split_cmdline(""), Some(Vec::new()));
    assert_eq!(split_cmdline("-m \"unclosed"), None);
    assert_eq!(split_cmdline("ends\\"), None);
}

/// `--autostash` of a merge or pull is read; Git's own concluding commit applies the stash.
#[test]
fn autostash_is_a_known_option_of_merge_and_pull() {
    assert!(merge_of(&["--autostash", "topic"]).is_some());
    assert!(merge_stops(words(&["--autostash", "--rebase", "origin"]).as_slice(), true).is_some());
}
