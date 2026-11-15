//! What:
//!  Controls of the words a removal refused for Git work prints,
//!  and of the one place a shell has to read them back from.
//! Why:
//!  The report hands the person a command to copy and run,
//!  so a path with a space,
//!  a quote,
//!  a dollar sign or a line break must reach that command as one word,
//!  and the raw bytes must not appear inside it
//!  (a review of `bab700404d`,
//!  finding 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(gitWorkRefusal(hostilePath, reasons)).toContain(`remove ${shellQuote(hostilePath)}`)
//! ```

use super::{Judged, answers_with, git_work_refusal, the_removal_it_judges};
use crate::forwarded_command::{ResolvedCommand, Route};
use std::path::{Path, PathBuf};

/// A path holding every hostile byte the review names,
/// in one word.
fn a_hostile_path() -> PathBuf {
    return PathBuf::from("/mnt/my task/it's \"quoted\" $HOME `id`\nafter");
}

/// What:
///  The refusal prints the opt-out with the path quoted as one word,
///  and not one byte of the raw path inside that command.
/// Why:
///  A shell splits the raw path into several words,
///  and a dollar sign or a backtick in it is expanded or run.
#[test]
fn the_refusal_quotes_the_path_it_prints() {
    let target: PathBuf = a_hostile_path();
    let report: String = git_work_refusal(target.as_path(), &[String::from("a reason")]);
    let quoted: String = crate::shell_quote::shell_quote(target.as_os_str().as_encoded_bytes());
    assert!(report.contains(format!("`git worktree --no-enforce-worktree remove {quoted}`").as_str()), "{report}");
    assert!(!report.contains("remove /mnt/my task/it's"), "the raw path is not inside the command: {report}");
    assert!(report.contains("a reason"), "the reason is said: {report}");
}

/// What:
///  A shell reads the quoted path back as the one word the removal needs.
/// Why:
///  The quoting rule is only worth its name if a shell turns the printed text back into the same path,
///  and this asks a shell rather than the function that produced it.
#[test]
fn a_shell_reads_the_printed_path_back_unchanged() {
    let target: PathBuf = a_hostile_path();
    let quoted: String = crate::shell_quote::shell_quote(target.as_os_str().as_encoded_bytes());
    let script: String = format!("printf %s {quoted}");
    let printed: std::process::Output = std::process::Command::new("/bin/sh").arg("-c").arg(script.as_str()).output().expect("a shell runs");
    assert!(printed.status.success(), "{}", String::from_utf8_lossy(printed.stderr.as_slice()));
    assert_eq!(String::from_utf8_lossy(printed.stdout.as_slice()), target.to_string_lossy());
}

/// What:
///  The words before the command are prose and name the worktree as the person reads it.
/// Why:
///  A person finds the worktree they typed by the same bytes in the sentence,
///  and the command beside it is the one that runs.
#[test]
fn the_sentence_names_the_worktree_as_it_stands() {
    let target: &Path = Path::new("/mnt/a plain path");
    let report: String = git_work_refusal(target, &[String::from("a reason")]);
    assert!(report.starts_with("monochromatic-git-policy-cli: /mnt/a plain path keeps Git work"), "{report}");
    assert!(report.contains("`git worktree --no-enforce-worktree remove '/mnt/a plain path'`"), "{report}");
}

/// What:
///  The words a `!` alias is judged by are the words its shell runs,
///  and the directory is the one Git starts that shell in.
/// Why:
///  For `alias.rmwt='!git worktree remove'` the caller's words start with `rmwt`,
///  so a guard reading them forwarded the removal past itself
///  (a review of `8858f843ee`, finding 1).
///  Git's own `rev-parse` names the top of the working tree, which is where the shell runs.
#[test]
fn a_shell_alias_is_judged_by_the_words_its_shell_runs() {
    let command: ResolvedCommand = ResolvedCommand {
        arguments: vec![std::ffi::OsString::from("rmwt"), std::ffi::OsString::from("../task")],
        subcommand_index: 0,
        subcommand: Some(std::ffi::OsString::from("rmwt")),
        route: Route::ShellAlias,
        shell_command: Some(vec![std::ffi::OsString::from("worktree"), std::ffi::OsString::from("remove"), std::ffi::OsString::from("../task")]),
        shell_value: Some(b"git worktree remove".to_vec()),
    };
    let judged: Judged = the_removal_it_judges(&command, &crate::test_support::git_context()).expect("the words can be read").expect("a removal was read");
    assert_eq!(judged.words, vec![std::ffi::OsString::from("worktree"), std::ffi::OsString::from("remove"), std::ffi::OsString::from("../task")]);
    assert_eq!(judged.subcommand_index, 0, "the subcommand stands at the front of the shell's words");
}

/// What:
///  A shell alias whose command this crate cannot read is a reason, whatever its text says.
/// Why:
///  A text a shell will run cannot be read to the end,
///  and nothing of this program can stand inside Git's shell: the wrapper never starts itself as a shim and never
///  writes shell text (`doc/planning/cli-git-no-self-invocation.md`).
///  So an alias this crate cannot read is not shown to be harmless
///  (a review of `746040080c`, findings 1 and 2).
#[test]
fn a_shell_alias_with_no_readable_git_command_is_left_to_git() {
    let command: ResolvedCommand = ResolvedCommand {
        arguments: vec![std::ffi::OsString::from("rmwt"), std::ffi::OsString::from("task")],
        subcommand_index: 0,
        subcommand: Some(std::ffi::OsString::from("rmwt")),
        route: Route::ShellAlias,
        shell_command: None,
        shell_value: None,
    };
    let refused: Result<Option<Judged>, String> = the_removal_it_judges(&command, &crate::test_support::git_context());
    match refused {
        Err(reason) => assert!(reason.contains("not one Git command line this program can read"), "and the reason says why: {reason}"),
        other => panic!("an alias whose command cannot be read is a reason, but it answered {other:?}"),
    }
}

/// What:
///  The options a Git this guard starts carries are the caller's configuration and nothing that moves it.
/// Why:
///  A forwarded relative `-C` was applied a second time, so the query failed and a valid removal was refused
///  (a review of `8858f843ee`, finding 4);
///  `-c core.ignorecase=true` still has to reach the question it decides.
#[test]
fn the_options_that_carry_are_configuration_and_never_movement() {
    let globals: Vec<std::ffi::OsString> = ["-C", "repository", "-c", "core.ignorecase=true", "--no-pager", "--git-dir", "/elsewhere/.git", "--config-env=core.abbrev=HOME"]
        .map(std::ffi::OsString::from)
        .to_vec();
    assert_eq!(
        answers_with(globals.as_slice()),
        vec![
            std::ffi::OsString::from("-c"),
            std::ffi::OsString::from("core.ignorecase=true"),
            std::ffi::OsString::from("--config-env=core.abbrev=HOME"),
        ]
    );
}

/// What:
///  A `!` alias whose Git command moves itself to another directory or names another repository is a reason.
/// Why:
///  The guard judges the repository the command was invoked in, and an inner `-C` or `--git-dir` names another one:
///  a look-up and the checks in the invoked repository, while real Git removed a worktree of the repository the inner
///  option named, was the hole
///  (a review of `746040080c`, finding 1).
///  So the alias is refused rather than judged against the wrong repository, and its opt-out runs it as real Git runs it.
#[test]
fn an_inner_global_option_is_part_of_the_command_that_is_judged() {
    let command: ResolvedCommand = ResolvedCommand {
        arguments: vec![std::ffi::OsString::from("rmwt"), std::ffi::OsString::from("task")],
        subcommand_index: 0,
        subcommand: Some(std::ffi::OsString::from("rmwt")),
        route: Route::ShellAlias,
        shell_command: Some(
            ["-C", "/elsewhere", "-c", "core.ignorecase=true", "worktree", "remove", "task"].map(std::ffi::OsString::from).to_vec(),
        ),
        shell_value: Some(b"git -C /elsewhere -c core.ignorecase=true worktree remove".to_vec()),
    };
    let refused: Result<Option<Judged>, String> = the_removal_it_judges(&command, &crate::test_support::git_context());
    match refused {
        Err(reason) => assert!(reason.contains("moves the removal to another directory"), "and the reason says which option did: {reason}"),
        other => panic!("an alias naming another repository is a reason, but it answered {other:?}"),
    }
    // A command line whose only own option is configuration is read and judged where it points.
    let configured: ResolvedCommand = ResolvedCommand {
        arguments: command.arguments.clone(),
        subcommand_index: command.subcommand_index,
        subcommand: command.subcommand.clone(),
        route: Route::ShellAlias,
        shell_command: Some(["-c", "core.ignorecase=true", "worktree", "remove", "task"].map(std::ffi::OsString::from).to_vec()),
        shell_value: Some(b"git -c core.ignorecase=true worktree remove".to_vec()),
    };
    let judged: Judged = the_removal_it_judges(&configured, &crate::test_support::git_context()).expect("the words can be read").expect("a removal was read");
    assert_eq!(judged.subcommand_index, 2, "the subcommand stands where Git's own layout puts it");
    assert_eq!(judged.words[2], std::ffi::OsString::from("worktree"), "and that word is the subcommand");
    assert_eq!(
        judged.answers_with,
        vec![std::ffi::OsString::from("-c"), std::ffi::OsString::from("core.ignorecase=true")],
        "and the inner configuration is what the guard's own Git carries"
    );
}
