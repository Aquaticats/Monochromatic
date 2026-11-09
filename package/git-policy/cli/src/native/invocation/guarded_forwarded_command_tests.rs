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
///  The caller's own words follow the alias value into the shell,
///  so both a `!git` alias and a `!git reset` alias are decided on the command that runs.
/// Why:
///  Git appends the caller's words to the value before it starts the shell
///  (`git.c:401` to `git.c:402`),
///  so
///  `git -c 'alias.wipe=!git' wipe reset --hard` and
///  `git -c 'alias.wipe=!git reset' wipe --hard`
///  both run a hard reset;
///  a reading that stopped at the value would decide about a command nobody runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['fixup', '!git']], words: ['fixup', 'reset', '--hard'] })
/// //   => decision 'needs-effective-target: reset-hard', arguments ['fixup', 'reset', '--hard']
/// ```
#[test]
fn a_shell_alias_is_decided_with_the_callers_arguments_after_its_own() {
    for (value, words, command) in [
        ("!git", vec!["fixup", "reset", "--hard"], GuardedCommand::ResetHard),
        ("!git reset", vec!["fixup", "--hard"], GuardedCommand::ResetHard),
        ("!git clean", vec!["fixup", "-fdx"], GuardedCommand::CleanIgnored),
    ] {
        let walked: GuardedForwarded = decided(&[("fixup", value)], words.as_slice());
        assert_eq!(
            walked.decision,
            LinkedWorktreeDecision::NeedsEffectiveTarget(command),
            "{value} {words:?}"
        );
        assert_eq!(walked.arguments, args(words.as_slice()), "{value}: Git receives the caller's own words");
    }
}

/// What the guard cannot read is left to Git and to the shell,
///  and no subcommand at all is still a pass.
#[test]
fn a_shell_alias_and_an_undefined_name_are_left_to_git() {
    for (pairs, words) in [
        // A shell command the plain reading cannot follow, and a command with a global option of its own.
        (vec![("fixup", "!sh -c 'git reset --hard'")], vec!["fixup"]),
        (vec![("fixup", "!git -C /elsewhere reset --hard")], vec!["fixup"]),
        (vec![], vec!["no-such-subcommand"]),
        (vec![], vec!["lfs", "pull"]),
        // An alias loop is Git's own error, not a guarded form.
        (vec![("a", "b"), ("b", "a")], vec!["a"]),
    ] {
        let walked: GuardedForwarded = decided(pairs.as_slice(), words.as_slice());
        assert_eq!(walked.decision, LinkedWorktreeDecision::Pass, "{words:?}");
        assert_eq!(walked.arguments, args(words.as_slice()), "{words:?}");
    }
    let none: GuardedForwarded =
        decide_guarded_forwarded(&context_with_aliases(&[]), no_environment().as_slice(), &[]);
    assert_eq!(none.decision, LinkedWorktreeDecision::Pass);
    assert!(none.arguments.is_empty());
}

/// What:
///  A shell alias that names one plain `git` command is decided on that command,
///  and the caller's own arguments are
///       what Git receives.
/// Why:
///   Git runs the shell with its own executable directory first on `PATH`,
///  so the `git` in that alias
///  is the real Git and this wrapper never sees the command run;
///  the guard has to read it from the value,
///  and
///  handing Git the shell unchanged keeps the environment Git gives it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['fixup', '!git reset --hard']], words: ['fixup'] })
/// //   => decision 'needs-effective-target: reset-hard', arguments ['fixup']
/// ```
#[test]
fn a_shell_alias_is_decided_on_the_git_command_it_names() {
    for (value, command) in [
        ("!git reset --hard", GuardedCommand::ResetHard),
        ("!git clean -fdx", GuardedCommand::CleanIgnored),
        ("!git restore --staged .", GuardedCommand::RestoreStaged),
    ] {
        let walked: GuardedForwarded = decided(&[("fixup", value)], &["fixup"]);
        assert_eq!(
            walked.decision,
            LinkedWorktreeDecision::NeedsEffectiveTarget(command),
            "{value}"
        );
        assert_eq!(walked.arguments, args(&["fixup"]), "{value}: Git receives the alias word");
    }
}
