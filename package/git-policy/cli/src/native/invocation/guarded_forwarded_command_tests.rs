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
use crate::test_support::{executable, fixture, git, isolated_overlay, remove};
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
    for (value, words, shell, command) in [
        (
            "!git",
            vec!["fixup", "reset", "--hard"],
            vec!["reset", "--hard"],
            GuardedCommand::ResetHard,
        ),
        (
            "!git reset",
            vec!["fixup", "--hard"],
            vec!["reset", "--hard"],
            GuardedCommand::ResetHard,
        ),
        (
            "!git clean",
            vec!["fixup", "-fdx"],
            vec!["clean", "-fdx"],
            GuardedCommand::CleanIgnored,
        ),
    ] {
        let walked: GuardedForwarded = decided(&[("fixup", value)], words.as_slice());
        assert_eq!(
            walked.decision,
            LinkedWorktreeDecision::NeedsEffectiveTarget(command),
            "{value} {words:?}"
        );
        assert_eq!(walked.arguments, args(words.as_slice()), "{value}: Git receives the caller's own words");
        assert_eq!(
            walked.decided_words,
            args(shell.as_slice()),
            "{value}: the rules read the command the shell runs"
        );
    }
}

/// What:
///  The words the guard decided about are the ones the rules read:
///  the command inside a `!` alias's shell,
///  an ordinary alias's expansion,
///  and the caller's own words for a built-in.
/// Why:
///  The flags and paths of a clean decide what it deletes,
///  and a rule that read the alias name instead saw a clean with no `-ffdX`
///  and no `-e` pattern:
///  it measured nothing,
///  saved nothing,
///  and Git deleted a nested repository and an untracked file unsaved.
///  A `!` alias carries the caller's words into the shell after its own (`git.c:401` to `git.c:402`),
///  so both halves are read here even though Git receives only the alias word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['spring', '!git clean -Xf']], words: ['spring', '-e', 'notes.txt'] })
/// //   => { decidedWords: ['clean', '-Xf', '-e', 'notes.txt'], arguments: ['spring', '-e', 'notes.txt'] }
/// ```
#[test]
fn the_words_the_guard_decided_about_are_the_ones_a_rule_reads() {
    let aliased: GuardedForwarded = decided(&[("spring", "!git clean -Xf")], &["spring", "-e", "notes.txt"]);
    assert_eq!(
        aliased.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::CleanIgnoredOnly)
    );
    assert_eq!(aliased.decided_words, args(&["clean", "-Xf", "-e", "notes.txt"]));
    assert_eq!(aliased.arguments, args(&["spring", "-e", "notes.txt"]));
    // An ordinary alias is replaced by its expansion, so the words Git receives are the words the rules read.
    let ordinary: GuardedForwarded = decided(&[("wipe", "reset --hard")], &["wipe"]);
    assert_eq!(ordinary.decided_words, args(&["reset", "--hard"]));
    assert_eq!(ordinary.decided_words, ordinary.arguments);
    // So is a command the caller names itself.
    let direct: GuardedForwarded = decided(&[], &["clean", "-ffdX"]);
    assert_eq!(direct.decided_words, args(&["clean", "-ffdX"]));
    assert_eq!(direct.decided_words, direct.arguments);
    // A shell the guard cannot read names no command, so it carries the caller's own words and changes nothing.
    let unread: GuardedForwarded = decided(&[("moves", "!cd sub && git clean -ffdX")], &["moves"]);
    assert_eq!(unread.decision, LinkedWorktreeDecision::Pass);
    assert_eq!(unread.decided_words, args(&["moves"]));
}

/// What the guard cannot read is left to Git and to the shell,
///  and no subcommand at all is still a pass.
#[test]
fn a_shell_alias_and_an_undefined_name_are_left_to_git() {
    for (pairs, words) in [
        // A shell command the plain reading cannot follow, and a command with a global option of its own.
        (vec![("fixup", "!sh -c 'git reset --hard'")], vec!["fixup"]),
        (vec![("fixup", "!git -C /elsewhere reset --hard")], vec!["fixup"]),
        // An alias loop is Git's own error, not a guarded form.
        (vec![("a", "b"), ("b", "a")], vec!["a"]),
    ] {
        let walked: GuardedForwarded = decided(pairs.as_slice(), words.as_slice());
        assert_eq!(walked.decision, LinkedWorktreeDecision::Pass, "{words:?}");
        assert_eq!(walked.arguments, args(words.as_slice()), "{words:?}");
    }
    // A word nothing defines is the one Git may replace with the command it guessed, so Git is told not to
    //  autocorrect this call and the person reads Git's own error and suggestion instead.
    let missing: GuardedForwarded = decided(&[], &["no-such-subcommand"]);
    assert_eq!(missing.decision, LinkedWorktreeDecision::Pass);
    assert_eq!(missing.arguments, args(&["-c", "help.autocorrect=0", "no-such-subcommand"]));
    assert_eq!(missing.decided_words, args(&["no-such-subcommand"]));
    // A name this host has no installed `git-<word>` for is that same case, which is why the host decides.
    let installed: GuardedForwarded = decided(&[], &["lfs", "pull"]);
    assert_eq!(installed.decision, LinkedWorktreeDecision::Pass);
    assert!(installed.arguments.ends_with(&args(&["lfs", "pull"])));
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

/// What:
///  A shell alias the caller moved into with `-C` is decided on the command the shell runs,
///  so an installed `git-<word>` below the directory the caller moved into hides nothing.
/// Why:
///  The owner's rule is that no invocation discards the person's data;
///  with `sub/bin/git-fixturewipe` present and `PATH` naming `bin` relative,
///  `git -C sub fixtureouter` runs `fixturewipe` from the top of the working tree,
///  where `bin` holds no program,
///  and the alias expands to `reset --hard`;
///  a guard that searched `sub/bin` found that program,
///  read the alias as an installed command,
///  and passed the invocation straight to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// decideGuardedForwarded({ pairs: [['fixtureouter', '!git fixturewipe'], ['fixturewipe', 'reset --hard']],
///   words: ['-C', 'sub', 'fixtureouter'], env: { PATH: 'bin:/usr/bin:/bin' } })
/// //   => decision 'needs-effective-target: reset-hard', arguments ['-C', 'sub', 'fixtureouter']
/// ```
#[test]
fn a_shell_alias_the_caller_moved_into_is_decided_where_git_runs_it() {
    let root: PathBuf = fixture("guard-shell-run-directory");
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main", "."]);
    let directory: PathBuf = root.join("sub/bin");
    std::fs::create_dir_all(directory.as_path()).expect("program directory");
    executable(directory.join("git-fixturewipe").as_path(), b"#!/bin/sh\necho external\n");
    let started_in: PathBuf = std::env::current_dir().expect("the directory this test started in");
    std::env::set_current_dir(root.as_path()).expect("the fixture is where the wrapper starts");
    let environment: Vec<(OsString, OsString)> = vec![(
        OsString::from("PATH"),
        OsString::from("bin:/usr/bin:/bin"),
    )];
    let walked: GuardedForwarded = decide_guarded_forwarded(
        &context_with_aliases(&[
            ("fixtureouter", "!git fixturewipe"),
            ("fixturewipe", "reset --hard"),
        ]),
        environment.as_slice(),
        &args(&["-C", "sub", "fixtureouter"]),
    );
    assert_eq!(
        walked.decision,
        LinkedWorktreeDecision::NeedsEffectiveTarget(GuardedCommand::ResetHard),
        "the guard read the command the shell runs"
    );
    assert_eq!(
        walked.arguments,
        args(&["-C", "sub", "fixtureouter"]),
        "Git receives the caller's own words"
    );
    std::env::set_current_dir(started_in.as_path()).expect("leave the fixture before removing it");
    remove(root.as_path());
}

/// What:
///  A `!` alias's shell runs at the top of the working tree,
///  so the decision carries that directory and not the one the caller stands in.
/// Why:
///  Git changes to the top of the working tree before it starts the shell of a `!` alias
///  (`git.c:388` to `git.c:407`),
///  so `git -c 'alias.shellcfg=!git config' shellcfg --file ../../named.config ...`
///  names one file from the caller's directory and another from the shell's.
///  A guard that resolved that relative path from the caller's directory judged a write
///  to `below/../..` while Git wrote the top's `../..`,
///  which is how a write to the configuration every worktree shares went through.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // cwd `<root>/below`; decideGuardedForwarded({ pairs: [['shellcfg', '!git config']], words: [...] })
/// //   => { decision: 'shared-remote', shellDirectory: root }
/// ```
#[test]
fn a_shell_alias_carries_the_directory_git_runs_its_shell_in() {
    let root: PathBuf = fixture("guard-shell-directory");
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main", "."]);
    let below: PathBuf = root.join("below");
    std::fs::create_dir_all(below.as_path()).expect("directory below the worktree root");
    let started_in: PathBuf = std::env::current_dir().expect("the directory this test started in");
    std::env::set_current_dir(below.as_path()).expect("the worktree is where the wrapper starts");
    let walked: GuardedForwarded = decide_guarded_forwarded(
        &context_with_aliases(&[("shellcfg", "!git config")]),
        no_environment().as_slice(),
        &args(&[
            "-c",
            "alias.shellcfg=!git config",
            "shellcfg",
            "--file",
            "../../named.config",
            "remote.origin.url",
            "https://example.invalid/through-a-shell-alias.git",
        ]),
    );
    assert!(
        matches!(walked.decision, LinkedWorktreeDecision::SharedRemote(_)),
        "the shell alias's write was not decided at all"
    );
    assert_eq!(
        walked.shell_directory.as_deref(),
        Some(root.as_path()),
        "the decision does not carry the directory the shell runs in"
    );
    std::env::set_current_dir(started_in.as_path()).expect("leave the fixture before removing it");
    remove(root.as_path());
}

/// What:
///  A command the caller names itself,
///  and one an ordinary alias expands to,
///  run where the caller's `-C` chain ends,
///  so neither carries a shell's directory.
/// Why:
///  The relative paths of those commands are read from the caller's directory,
///  and a decision that carried a shell's directory would refuse a write to a file of
///  the command's own,
///  or name the wrong shared file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // decideGuardedForwarded({ words: ['config', '--file', 'named.config', ...] }).shellDirectory === undefined
/// ```
#[test]
fn a_command_the_caller_names_carries_no_shell_directory() {
    for (pairs, words) in [
        (
            vec![],
            vec!["config", "--file", "named.config", "remote.origin.url", "https://example.invalid/named.git"],
        ),
        (
            vec![("cfg", "config")],
            vec!["cfg", "--file", "named.config", "remote.origin.url", "https://example.invalid/named.git"],
        ),
    ] {
        let walked: GuardedForwarded = decided(pairs.as_slice(), words.as_slice());
        assert!(
            matches!(walked.decision, LinkedWorktreeDecision::SharedRemote(_)),
            "{words:?} was not decided at all"
        );
        assert_eq!(walked.shell_directory, None, "{words:?} carries a shell directory");
    }
}
