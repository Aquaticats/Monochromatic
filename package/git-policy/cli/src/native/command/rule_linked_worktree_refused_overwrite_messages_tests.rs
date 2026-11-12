//! What:
//!  The words of the refusal that names the entries a discarding command would lose.
//! Why:
//!  These are the only words a person whose file is at stake has:
//!  they must name the entries,
//!  say why Git would lose them,
//!  say that nothing ran,
//!  and print the command that runs it anyway,
//!  ready to paste.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // overwrittenRefusalMessage({ command: 'reset-hard', region: ['--hard'], paths: ['b.txt'] })
//! ```

use super::{Overwritten, overwritten_refusal_message};
use crate::command_test_support::os_arguments;
use crate::rule_linked_worktree::GuardedCommand;
use std::ffi::OsString;

/// The refusal of one form over one entry.
fn refusal(command: GuardedCommand, region: &[&str], paths: &[&str]) -> String {
    let tokens: Vec<OsString> = os_arguments(region);
    let named: Vec<String> = paths.iter().map(|path| return (*path).to_string()).collect();
    return overwritten_refusal_message(Overwritten {
        command,
        region: tokens.as_slice(),
        paths: named.as_slice(),
    });
}

/// One entry is named,
/// the refusal says why it stands nowhere else,
/// that nothing ran,
/// and the exact command that runs it anyway.
#[test]
fn one_entry_is_named_with_the_command_that_runs_it_anyway() {
    let message: String = refusal(GuardedCommand::ResetHard, &["--hard", "other"], &["b.txt"]);
    assert!(message.contains("git reset is rejected"), "{message}");
    assert!(message.contains("`b.txt`, which stands nowhere else"), "{message}");
    assert!(message.contains("Git did not put it in this worktree"), "{message}");
    assert!(message.contains("Nothing in your worktree was changed"), "{message}");
    assert!(
        message.contains("run `git reset --no-enforce-worktree --hard other`"),
        "{message}"
    );
}

/// Several entries are named,
/// and the sentence speaks of them as more than one.
#[test]
fn several_entries_are_named_and_spoken_of_as_more_than_one() {
    let message: String = refusal(
        GuardedCommand::CheckoutPaths,
        &["--force", "other"],
        &["b.txt", "nested/", "other.txt"],
    );
    assert!(message.contains("`b.txt`, `nested/`, `other.txt`, which stand nowhere else"), "{message}");
    assert!(message.contains("Git did not put them in this worktree"), "{message}");
    assert!(message.contains("run `git checkout --no-enforce-worktree --force other`"), "{message}");
}

/// A list longer than the bound names the first entries and counts the rest,
/// so a tree holding thousands of them still prints a sentence.
#[test]
fn a_long_list_is_named_and_counted() {
    let paths: Vec<String> = (0..12).map(|number| return format!("file-{number}.txt")).collect();
    let borrowed: Vec<&str> = paths.iter().map(|path| return path.as_str()).collect();
    let message: String = refusal(GuardedCommand::ReadTree, &["-m", "-u", "HEAD", "other"], borrowed.as_slice());
    assert!(message.contains("`file-0.txt`, `file-1.txt`, `file-2.txt`, `file-3.txt`, `file-4.txt`"), "{message}");
    assert!(!message.contains("file-5.txt"), "{message}");
    assert!(message.contains("and 7 more"), "{message}");
    assert!(message.contains("run `git read-tree --no-enforce-worktree -m -u HEAD other`"), "{message}");
}

/// A name that would break a pasted line is quoted,
/// and a form with no words after it still prints a command.
#[test]
fn a_name_with_a_space_is_quoted_and_an_empty_region_still_prints_a_command() {
    let message: String = refusal(GuardedCommand::RestoreWorktree, &[], &["a file with spaces.txt"]);
    assert!(message.contains("run `git restore --no-enforce-worktree`"), "{message}");
    let spaced: String = refusal(GuardedCommand::Remove, &["--force", "a b.txt"], &["a b.txt"]);
    assert!(spaced.contains("run `git rm --no-enforce-worktree --force 'a b.txt'`"), "{spaced}");
}

/// An `--source <tree>` value that looks like a flag stays one word of the pasted command.
#[test]
fn a_value_that_looks_like_a_flag_is_quoted_into_the_pasted_command() {
    let message: String = refusal(GuardedCommand::RestoreWorktree, &["--source", "other", "--", "b.txt"], &["b.txt"]);
    assert!(
        message.contains("run `git restore --no-enforce-worktree --source other -- b.txt`"),
        "{message}"
    );
}
