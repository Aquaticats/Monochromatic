//! What:
//!  Controls for the guarded decision about the command Git really runs:
//!  a built-in decided without a configuration read,
//!  an alias decided on its expansion,
//!  a chained alias,
//!  a `!` shell alias forwarded as it is and decided by nothing,
//!  and an undefined name left to Git.
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

/// What:
///  An environment naming no `PATH`,
///  so no executable of the test process answers for an installed command.
/// Why:
///   The guard asks the same search the resolver asks,
///  and a control sets every variable it depends on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noEnvironment = [];
/// ```
fn no_environment() -> Vec<(OsString, OsString)> {
    return Vec::new();
}

/// Decide one invocation and return its decision together with the arguments Git will run.
fn decided(pairs: &[(&str, &str)], words: &[&str]) -> GuardedForwarded {
    return decide_guarded_forwarded(&context_with_aliases(pairs), no_environment().as_slice(), args(words).as_slice());
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
    // An expansion that moves the directory is refused by Git before it runs anything, so the guard leaves it to Git:
    //  no fact of the caller describes a repository the command never runs in.
    let moved: GuardedForwarded = decided(&[("sweep", "-C dir clean -fdx")], &["sweep"]);
    assert_eq!(moved.decision, LinkedWorktreeDecision::Pass);
    assert_eq!(moved.arguments, args(&["sweep"]));
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

/// What:
///  Every `!` shell alias is forwarded as Git receives it and judged by nothing.
/// Why:
///  The owner decided on 2026-11-15 that every `!` alias is passed to real Git as it is,
///  with all aliases working and no opt-out
///  (`doc/handover/cli-git-rust-implementation.md`, "Answers on waking, second round" and
///  "third round"):
///  a shell command cannot be judged by reading it,
///  and the `git` inside it is real Git, not this wrapper.
///  So `git <alias>` where the shell ends in a guarded command is a hole,
///  which `SPEC.md` states,
///  and a control that decided one of these would be the refusal the decision removed.
///  The rows include the value the guard used to read as one plain command (`!git reset --hard`)
///  and the caller's own words Git appends to it (`!git` with `reset --hard` after it).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['fixup', '!git reset --hard']], words: ['fixup'] })
/// //   => { decision: 'pass', args: ['fixup'] }
/// ```
#[test]
fn a_shell_alias_is_forwarded_as_it_is_and_judged_by_nothing() {
    for (pairs, words) in [
        (vec![("fixup", "!git reset --hard")], vec!["fixup"]),
        (vec![("fixup", "!git reset")], vec!["fixup", "--hard"]),
        (vec![("fixup", "!git")], vec!["fixup", "reset", "--hard"]),
        (vec![("fixup", "!git clean -fdx")], vec!["fixup"]),
        (vec![("fixup", "!git restore --staged .")], vec!["fixup"]),
        (vec![("fixup", "!git config")], vec!["fixup", "--file", "named.config", "remote.origin.url", "https://example.invalid/named.git"]),
        (vec![("fixup", "!sh -c 'git reset --hard'")], vec!["fixup"]),
        (vec![("fixup", "!git -C /elsewhere reset --hard")], vec!["fixup"]),
        // A chain of shell aliases is Git's own business, and no link of it is a guarded form here.
        (vec![("a", "!git b"), ("b", "!git reset --hard")], vec!["a"]),
    ] {
        let walked: GuardedForwarded = decided(pairs.as_slice(), words.as_slice());
        assert_eq!(walked.decision, LinkedWorktreeDecision::Pass, "{words:?}");
        assert_eq!(walked.arguments, args(words.as_slice()), "{words:?}: Git receives the caller's own words");
    }
}

/// What:
///  An ordinary alias and a word the caller names are decided on the words Git receives,
///  which are the same list the guard reads.
/// Why:
///  An ordinary alias is replaced by its expansion before Git starts,
///  so the flags and paths Git will act on are the expansion's,
///  and a built-in word is itself.
///  That one list is what the transforms add their tokens to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['wipe', 'reset --hard']], words: ['wipe'] }).args === ['reset', '--hard'];
/// ```
#[test]
fn an_alias_and_a_builtin_are_decided_on_the_words_git_receives() {
    let ordinary: GuardedForwarded = decided(&[("wipe", "reset --hard")], &["wipe"]);
    assert_eq!(ordinary.arguments, args(&["reset", "--hard"]));
    let direct: GuardedForwarded = decided(&[], &["clean", "-ffdX"]);
    assert_eq!(direct.arguments, args(&["clean", "-ffdX"]));
    assert_eq!(
        direct.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnoredOnly)
    );
}

/// What the guard cannot read is left to Git and to the shell,
///  and no subcommand at all is still a pass.
#[test]
fn an_undefined_name_is_left_to_git() {
    // A word nothing defines is the one Git may replace with the command it guessed, so Git is told not to
    //  autocorrect this call and the person reads Git's own error and suggestion instead.
    let missing: GuardedForwarded = decided(&[], &["no-such-subcommand"]);
    assert_eq!(missing.decision, LinkedWorktreeDecision::Pass);
    assert_eq!(missing.arguments, args(&["-c", "help.autocorrect=0", "no-such-subcommand"]));
    // A name this host has no installed `git-<word>` for is that same case, which is why the host decides.
    let installed: GuardedForwarded = decided(&[], &["lfs", "pull"]);
    assert_eq!(installed.decision, LinkedWorktreeDecision::Pass);
    assert!(installed.arguments.ends_with(&args(&["lfs", "pull"])));
    let none: GuardedForwarded =
        decide_guarded_forwarded(&context_with_aliases(&[]), no_environment().as_slice(), &[]);
    assert_eq!(none.decision, LinkedWorktreeDecision::Pass);
    assert!(none.arguments.is_empty());
}
