//! What:
//!  Controls for the guarded decision about the command Git really runs:
//!  a built-in decided without a configuration read,
//!  an alias decided on its expansion,
//!  a chained alias,
//!  and a shell alias or undefined name left to Git.
//! Why:
//!  The word on the command line hid every guarded form from the guard,
//!  and normal use must keep working when the expansion is harmless.

use super::{GuardedForwarded, decide_guarded_forwarded};
use crate::rule_linked_worktree::{GuardedCommand, LinkedWorktreeDecision};
use crate::test_support::isolated_overlay;
use crate::transaction_git::GitContext;
use std::ffi::OsString;
use std::path::PathBuf;

/// A context whose only aliases are the given name and value pairs,
///  read by real Git through its `GIT_CONFIG_*` overlay.
fn context_with_aliases(pairs: &[(&str, &str)]) -> GitContext {
    let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
    overlay.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from(pairs.len().to_string())));
    for (index, (name, value)) in pairs.iter().enumerate() {
        overlay.push((
            OsString::from(format!("GIT_CONFIG_KEY_{index}")),
            OsString::from(format!("alias.{name}")),
        ));
        overlay.push((
            OsString::from(format!("GIT_CONFIG_VALUE_{index}")),
            OsString::from(*value),
        ));
    }
    return GitContext {
        real_git: PathBuf::from(crate::test_support::REAL_GIT),
        overlay,
        global_prefix: Vec::new(),
        common_dir: None,
    };
}

/// Arguments as the wrapper receives them.
fn args(words: &[&str]) -> Vec<OsString> {
    return words.iter().map(OsString::from).collect();
}

/// Decide one invocation and return its decision together with the arguments Git will run.
fn decided(pairs: &[(&str, &str)], words: &[&str]) -> GuardedForwarded {
    return decide_guarded_forwarded(&context_with_aliases(pairs), args(words).as_slice());
}

/// A built-in word is decided as it always was,
///  with no configuration read,
///  and its arguments are untouched.
#[test]
fn a_builtin_word_is_decided_without_an_alias_lookup() {
    for (words, command) in [
        (
            vec!["reset", "--hard"],
            LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetHard),
        ),
        (vec!["status", "--short"], LinkedWorktreeDecision::Pass),
        (
            vec!["-C", "dir", "clean", "-fdx"],
            LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnored),
        ),
    ] {
        let walked: GuardedForwarded = decided(&[], words.as_slice());
        assert_eq!(walked.decision, command, "{words:?}");
        assert_eq!(walked.arguments, args(words.as_slice()), "{words:?}");
    }
}

/// An alias is decided on the command it expands to,
///  and the arguments are the expansion Git will run.
#[test]
fn an_alias_is_decided_on_the_command_it_expands_to() {
    let wiped: GuardedForwarded = decided(&[("wipe", "reset --hard")], &["wipe"]);
    assert_eq!(
        wiped.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetHard)
    );
    assert_eq!(wiped.arguments, args(&["reset", "--hard"]));
    // An alias whose expansion carries a global option keeps it in front.
    let cleaned: GuardedForwarded = decided(&[("sweep", "-C dir clean -fdx")], &["sweep"]);
    assert_eq!(
        cleaned.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnored)
    );
    assert_eq!(cleaned.arguments, args(&["-C", "dir", "clean", "-fdx"]));
    // A harmless expansion runs, with the expansion as the arguments Git receives.
    let harmless: GuardedForwarded = decided(&[("short", "status --short")], &["short"]);
    assert_eq!(harmless.decision, LinkedWorktreeDecision::Pass);
    assert_eq!(harmless.arguments, args(&["status", "--short"]));
    // A chained alias is followed to the end.
    let chained: GuardedForwarded = decided(&[("a", "b"), ("b", "reset --hard")], &["a"]);
    assert_eq!(
        chained.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetHard)
    );
    assert_eq!(chained.arguments, args(&["reset", "--hard"]));
}

/// What the guard cannot read is left to Git and to the shell,
///  and no subcommand at all is still a pass.
#[test]
fn a_shell_alias_and_an_undefined_name_are_left_to_git() {
    for (pairs, words) in [
        (vec![("fixup", "!git reset --hard")], vec!["fixup"]),
        (vec![], vec!["no-such-subcommand"]),
        (vec![], vec!["lfs", "pull"]),
        // An alias loop is Git's own error, not a guarded form.
        (vec![("a", "b"), ("b", "a")], vec!["a"]),
    ] {
        let walked: GuardedForwarded = decided(pairs.as_slice(), words.as_slice());
        assert_eq!(walked.decision, LinkedWorktreeDecision::Pass, "{words:?}");
        assert_eq!(walked.arguments, args(words.as_slice()), "{words:?}");
    }
    let none: GuardedForwarded = decide_guarded_forwarded(&context_with_aliases(&[]), &[]);
    assert_eq!(none.decision, LinkedWorktreeDecision::Pass);
    assert!(none.arguments.is_empty());
}
