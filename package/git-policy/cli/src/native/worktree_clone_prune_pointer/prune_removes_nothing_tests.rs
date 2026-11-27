//! Controls for whether one `git worktree prune` removes nothing:
//! every spelling Git's own parser accepts for the dry run and its negation,
//! the forms Git refuses before it prunes,
//! and the words this reading leaves to the guard.

use super::*;
use crate::command_test_support::os_arguments;

/// The answer for one `git worktree prune` line,
/// given the words after `worktree`.
fn removes_nothing(values: &[&str]) -> bool {
    let words: Vec<OsString> = os_arguments(values);
    return a_prune_that_removes_nothing(words.as_slice());
}

/// What:
///  Every spelling Git accepts for the dry run removes nothing.
/// Why:
///  Git matches a unique prefix of `--dry-run` and the short `-n` inside a
///  cluster of short options,
///  so a pass that read only the exact words wrote the pointer on a command
///  that pruned nothing.
#[test]
fn every_spelling_of_the_dry_run_removes_nothing() {
    for values in [
        &["prune", "-n"][..],
        &["prune", "-nv"],
        &["prune", "-vn"],
        &["prune", "-nvn"],
        &["prune", "--dry-run"],
        &["prune", "--d"],
        &["prune", "--dr"],
        &["prune", "--dry"],
        &["prune", "--dry-"],
        &["prune", "--dry-r"],
        &["prune", "--dry-ru"],
        &["prune", "-n", "--dry-run"],
        &["prune", "--dry-run", "-n"],
    ] {
        assert!(removes_nothing(values), "Git prunes nothing for {values:?}");
    }
}

/// What:
///  The last dry-run word decides,
///  so Git's negation and every abbreviation of it is a real prune.
/// Why:
///  Git applies the words in the order the person typed them,
///  and reading any dry-run word as final would skip the guard on a command
///  that drops registrations.
#[test]
fn the_last_word_decides_and_the_negation_is_a_real_prune() {
    for values in [
        &["prune"][..],
        &["prune", "--no-dry-run"],
        &["prune", "--no-d"],
        &["prune", "--no-dr"],
        &["prune", "--no-dru"],
        &["prune", "--dry-run", "--no-dry-run"],
        &["prune", "-n", "--no-d"],
        &["prune", "--expire", "now"],
        &["prune", "--expire=now"],
        &["prune", "--e", "now"],
        &["prune", "--verbose"],
    ] {
        assert!(!removes_nothing(values), "Git prunes for {values:?}");
    }
    assert!(
        removes_nothing(&["prune", "--no-dry-run", "--dry"]),
        "the last word is the dry run again"
    );
    assert!(
        removes_nothing(&["prune", "--no-dry-r", "-n"]),
        "the short dry run after Git's negation"
    );
}

/// What:
///  A form Git refuses before it prunes removes nothing,
///  and a word this reading does not know is left to the guard.
/// Why:
///  Git answers `--dry-run=<value>` and any operand with an error rather than
///  a prune,
///  while an option this reading does not know must never be taken for a dry
///  run,
///  because a spelling a later Git adds would then skip the guard.
#[test]
fn a_refused_form_removes_nothing_and_an_unknown_option_is_guarded() {
    for values in [
        &["prune", "--dry-run=1"][..],
        &["prune", "--no-dry-run=0"],
        &["prune", "one"],
        &["prune", "--", "one"],
        &["prune", "-n", "one"],
    ] {
        assert!(
            removes_nothing(values),
            "Git refuses this form and prunes nothing: {values:?}"
        );
    }
    for values in [
        &["prune", "--frobnicate"][..],
        &["prune", "-x"],
        &["prune", "--verbose", "--frobnicate"],
    ] {
        assert!(
            !removes_nothing(values),
            "an option this reading does not know still goes through the guard: {values:?}"
        );
    }
    assert!(
        !removes_nothing(&["--verbose", "prune"]),
        "the subcommand word is the first word that is no option"
    );
}
