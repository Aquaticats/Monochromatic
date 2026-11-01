//! Controls for the commit argument transforms, including the incumbent's quirks.

use super::*;

/// Builds owned argument lists from string literals.
fn args(items: &[&str]) -> Vec<String> {
    return items.iter().map(|item| { return String::from(*item); }).collect();
}

/// A cluster carrying a value is split into flags and the value token; a pathspec tail is kept verbatim.
#[test]
fn normalise_splits_inline_values_and_keeps_the_tail() {
    assert_eq!(normalise_commit_args(&args(&["-amhello"])), args(&["-a", "-m", "hello"]));
    assert_eq!(normalise_commit_args(&args(&["-m", "x"])), args(&["-m", "x"]));
    assert_eq!(normalise_commit_args(&args(&["--message=x"])), args(&["--message=x"]));
    assert_eq!(
        normalise_commit_args(&args(&["-m", "x", "--", "-mfoo"])),
        args(&["-m", "x", "--", "-mfoo"])
    );
    assert_eq!(normalise_commit_args(&args(&["-mé"])), args(&["-m", "é"]));
}

/// Global options that redirect Git are dropped with their values; other globals stay in order.
#[test]
fn split_drops_redirecting_globals_and_keeps_the_rest() {
    let input: Vec<String> = args(&[
        "-C", "r", "-c", "a.b=1", "--git-dir=g", "--bare", "commit", "-m", "x",
    ]);
    let (globals, commit) = split_commit_invocation(&input, 6);
    assert_eq!(globals, args(&["-c", "a.b=1"]));
    assert_eq!(commit, args(&["-m", "x"]));
}

/// `-a` leaves the region, a cluster keeps its other letters, and a value-taking letter stays with its value.
#[test]
fn without_all_flag_removes_only_the_all_letter() {
    assert_eq!(without_all_flag(&args(&["-am", "x"])), args(&["-m", "x"]));
    assert_eq!(without_all_flag(&args(&["-a", "-m", "x"])), args(&["-m", "x"]));
    assert_eq!(without_all_flag(&args(&["--all"])), Vec::<String>::new());
    assert_eq!(without_all_flag(&args(&["-aSkey"])), args(&["-Skey"]));
    assert_eq!(without_all_flag(&args(&["-a"])), Vec::<String>::new());
}

/// A value-taking option's value is kept even when it spells `--all`, as the incumbent keeps it.
#[test]
fn without_all_flag_keeps_an_all_spelled_as_a_message() {
    assert_eq!(without_all_flag(&args(&["-m", "--all"])), args(&["-m", "--all"]));
}

/// After the pathspec separator, every token stays, even `-a`.
#[test]
fn without_all_flag_keeps_everything_after_the_separator() {
    assert_eq!(
        without_all_flag(&args(&["-m", "x", "--", "-a"])),
        args(&["-m", "x", "--", "-a"])
    );
}

/// The explicit-path removal drops the last occurrence of each pathspec, so a message equal to a
///  pathspec is lost when it is the later token (the incumbent's `lastIndexOf` behavior).
#[test]
fn explicit_removal_takes_the_last_occurrence_of_each_pathspec() {
    assert_eq!(
        private_explicit_commit_args(&args(&["a.txt", "-m", "a.txt"]), &args(&["a.txt"])),
        args(&["a.txt", "-m"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "a.txt", "a.txt"]), &args(&["a.txt"])),
        args(&["-m", "a.txt"])
    );
}

/// The pathspec-file controls, the separator and `--only` never reach the real commit.
#[test]
fn explicit_removal_drops_pathspec_file_controls_and_only() {
    assert_eq!(
        private_explicit_commit_args(
            &args(&["--pathspec-from-file=f", "--pathspec-file-nul", "-m", "x"]),
            &[],
        ),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["--pathspec-from-file", "f", "-m", "x"]), &[]),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_explicit_commit_args(&args(&["-m", "x", "--only", "--", "a"]), &args(&["a"])),
        args(&["-m", "x"])
    );
}

/// Selection controls go away once the private index already holds the choice.
#[test]
fn selection_removal_drops_selection_controls() {
    assert_eq!(
        private_selection_commit_args(&args(&["--patch", "-m", "x", "a"]), &args(&["a"])),
        args(&["-m", "x"])
    );
    assert_eq!(
        private_selection_commit_args(&args(&["-i", "-m", "x"]), &[]),
        args(&["-m", "x"])
    );
}

/// The real commit arguments depend on the mode and on whether selection ran privately.
#[test]
fn resolve_chooses_the_arguments_by_mode() {
    let input: Vec<String> = args(&["--patch", "-m", "x", "a"]);
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::ExplicitPath, false),
        args(&["--patch", "-m", "x"])
    );
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::Index, true),
        args(&["-m", "x"])
    );
    assert_eq!(
        resolve_private_commit_args(&input, &args(&["a"]), TransactionMode::Index, false),
        input
    );
}
