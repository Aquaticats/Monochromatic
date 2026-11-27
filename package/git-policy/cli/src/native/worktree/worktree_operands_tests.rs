//! Controls for the operands one `git worktree` subcommand was given:
//! the plain form,
//! every option spelling Git accepts before the operand,
//! the separator,
//! and the two words Git reads as operands despite a leading `-`.

use super::*;
use crate::command_test_support::os_arguments;

/// The operands of one argument list,
/// as the words a person typed.
fn read(values: &[&str]) -> Vec<String> {
    let arguments: Vec<OsString> = os_arguments(values);
    let operands: Vec<&OsString> = operands_after_options(arguments.as_slice());
    return operands
        .iter()
        .map(|operand| return operand.to_string_lossy().into_owned())
        .collect();
}

/// What:
///  The plain form and every option spelling leave the worktree as the one
///  operand.
/// Why:
///  Git's parser drops each of these before the subcommand sees its arguments,
///  and a reading that took an option for the worktree would name the wrong
///  path.
#[test]
fn the_options_leave_the_worktree_as_the_operand() {
    for (values, expected) in [
        (&["one"][..], &["one"][..]),
        (&["-f", "one"], &["one"]),
        (&["-f", "-f", "one"], &["one"]),
        (&["-ff", "one"], &["one"]),
        (&["--force", "one"], &["one"]),
        (&["--force", "--force", "one"], &["one"]),
        (&["--force", "-f", "one"], &["one"]),
        (&["--forc", "one"], &["one"]),
        (&["--", "one"], &["one"]),
        (&["-f", "--", "one"], &["one"]),
        (&["--force", "--force", "--", "one"], &["one"]),
        (&["-f", "one", "two"], &["one", "two"]),
    ] {
        assert_eq!(read(values), expected, "{values:?}");
    }
}

/// What:
///  A word after the separator is an operand even when it starts with `-`,
///  and a word of one `-` is an operand wherever it stands.
/// Why:
///  Git reads both as names,
///  so a scan that kept skipping words by their first character would lose the
///  worktree a person named.
#[test]
fn the_separator_and_a_single_dash_keep_words_as_operands() {
    assert_eq!(
        read(&["--", "-f", "--force"]),
        vec!["-f", "--force"],
        "every word after the separator is an operand"
    );
    assert_eq!(
        read(&["-", "one"]),
        vec!["-", "one"],
        "a word of one `-` names a worktree rather than an option"
    );
}

/// What:
///  A subcommand given no words has no operand.
/// Why:
///  The scan answers what it was given rather than inventing a name,
///  and Git answers its usage for such a command.
#[test]
fn no_words_are_no_operands() {
    assert!(read(&[]).is_empty());
    assert!(read(&["--force"]).is_empty());
    assert!(read(&["--"]).is_empty());
}
