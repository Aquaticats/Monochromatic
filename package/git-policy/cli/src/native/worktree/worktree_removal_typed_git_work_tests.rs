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

use super::{answers_with, git_work_refusal, the_removal_it_judges};
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
///  A `!` alias is forwarded as it is, whatever its own command line says.
/// Why:
///  The owner's decision of 2026-11-15: "Forward them unguarded".
///  Every `!` alias is passed to real Git as it is, a removal it runs is not guarded, and `SPEC.md` states that hole.
///  A reader of a shell's text cannot follow it to the end, and the wrapper never starts itself as a shim and never
///  writes shell text (`doc/planning/cli-git-no-self-invocation.md`).
#[test]
fn every_shell_alias_is_forwarded_as_it_is() {
    for shell in [
        Some(vec![std::ffi::OsString::from("worktree"), std::ffi::OsString::from("remove"), std::ffi::OsString::from("task")]),
        Some(["-C", "/elsewhere", "worktree", "remove", "task"].map(std::ffi::OsString::from).to_vec()),
        Some(["-c", "core.ignorecase=true", "worktree", "remove", "task"].map(std::ffi::OsString::from).to_vec()),
        None,
    ] {
        let command: ResolvedCommand = ResolvedCommand {
            arguments: vec![std::ffi::OsString::from("rmwt"), std::ffi::OsString::from("task")],
            subcommand_index: 0,
            subcommand: Some(std::ffi::OsString::from("rmwt")),
            route: Route::ShellAlias,
            shell_command: shell.clone(),
            shell_value: Some(b"git worktree remove".to_vec()),
        };
        assert!(
            matches!(the_removal_it_judges(&command, &crate::test_support::git_context()), Ok(None)),
            "a shell alias is no command this guard judges: {shell:?}",
        );
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
