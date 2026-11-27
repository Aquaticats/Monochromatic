//! What:
//! The test that checks which program each printed sentence names:
//! every sentence the two scopes print goes through `command_naming`,
//! the command it prints must start with the program that scope's install put on `PATH`,
//! and a shell in the fixture must find that program.
//! Why:
//! Measured on 2026-11-11 in a home laid out like this machine's:
//! the machine-global plan declares no `mpm-shadow` entry,
//! so every sentence naming `mpm-shadow` failed with
//! `/bin/sh: line 1: mpm-shadow: command not found`,
//! exit status 127,
//! and nothing but a worktree's `PATH` ever makes `mpm` the shadow.
//! A sentence may print other backticked words before its command,
//! so the program is what identifies the command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // printed_remedy/every_sentence_names_a_program_the_install_puts_on_path.test.ts
//! for (const sentence of sentences) {
//!   const command = commandNaming(sentence, shadowProgram);
//!   expect(shell(`command -v ${shadowProgram}`).status).toBe(0);
//! }
//! ```

use std::path::PathBuf;

use mpm_shadow::toolchain::cargo_installs::older_lock_message_for;
use mpm_shadow::toolchain::launcher_current_link::{
    MissingProgram,
    missing_program_advice,
};
use mpm_shadow::toolchain::scope::{
    LOCK_FILE_NAME,
    ScopeKind,
};

use crate::a_machine_with_both_scopes_installed::{
    Machine,
    Place,
    SHADOW_PROGRAM,
};
use crate::{
    INSTALLED,
    LOCKED,
    TOOL_ID,
    text,
};

/// The name the repository plan's postinstall link gives `mpm-shadow` in a worktree,
/// the program the tree's sentences name.
const SHIM_PROGRAM: &str = "mpm";

/// The program a printed command starts with,
/// the word a shell would look up on `PATH`.
fn command_program(command: &str) -> &str {
    return command.split(' ').next().unwrap_or_default();
}

/// What:
/// The command a sentence prints that starts with `program`.
/// Why:
/// A sentence may print other backticked words (`current`,
/// a lock file)
/// before the command it tells a person to run,
/// so the program is what identifies the command,
/// and a sentence that names a different program than the one expected fails here.
fn command_naming(
    sentence: &str,
    program: &str,
) -> String {
    let mut rest: &str = sentence;
    while let Some(open) = rest.find('`') {
        let after: &str = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let span: &str = &after[..close];
        let first: &str = span.split(' ').next().unwrap_or_default();
        if first == program {
            return span.to_string();
        }
        rest = &after[close + 1..];
    }
    panic!("the sentence prints no command starting with {program}: {sentence}");
}

#[test]
fn every_printed_remedy_names_a_program_its_own_install_puts_on_path() {
    let machine = Machine::new();
    let installed = machine.install_global();
    assert_eq!(
        installed.status.code(),
        Some(0),
        "the machine-global install failed:\n{}",
        text(&installed.stderr)
    );
    let linked = machine.install_repo();
    assert_eq!(
        linked.status.code(),
        Some(0),
        "the worktree's install failed:\n{}",
        text(&linked.stderr)
    );

    let global_lock: PathBuf = machine.global_lock_file();
    let missing_global: PathBuf = machine.farm().join("nothing-here");
    let global_sentences: [String; 2] = [
        older_lock_message_for(ScopeKind::Global, TOOL_ID, LOCKED, INSTALLED, &global_lock),
        missing_program_advice(MissingProgram {
            tool_id: TOOL_ID,
            scope: ScopeKind::Global,
            lock_file: &global_lock,
            real: &missing_global,
            lock_text: None,
        }),
    ];
    for sentence in &global_sentences {
        let command: String = command_naming(sentence, SHADOW_PROGRAM);
        assert_eq!(
            command_program(&command),
            SHADOW_PROGRAM,
            "the sentence starts its command with the program it needs: {sentence}"
        );
        let found = machine.shell(&format!("command -v {SHADOW_PROGRAM}"), Place::Global);
        assert!(
            found.status.success(),
            "the machine-global install puts `{SHADOW_PROGRAM}` on the machine's PATH, so `{command}` runs as printed: {sentence}"
        );
    }

    let repo_lock: PathBuf = machine.repo().join(LOCK_FILE_NAME);
    let missing_repo: PathBuf = machine
        .repo()
        .join(".cache")
        .join("toolchain-bin")
        .join("nothing-here");
    let repo_sentences: [String; 2] = [
        older_lock_message_for(ScopeKind::Repo, TOOL_ID, LOCKED, INSTALLED, &repo_lock),
        missing_program_advice(MissingProgram {
            tool_id: TOOL_ID,
            scope: ScopeKind::Repo,
            lock_file: &repo_lock,
            real: &missing_repo,
            lock_text: None,
        }),
    ];
    for sentence in &repo_sentences {
        let command: String = command_naming(sentence, SHIM_PROGRAM);
        assert_eq!(
            command_program(&command),
            SHIM_PROGRAM,
            "the sentence starts its command with the program it needs: {sentence}"
        );
        let found = machine.shell(&format!("command -v {SHIM_PROGRAM}"), Place::Repo);
        assert!(
            found.status.success(),
            "the worktree's install puts `{SHIM_PROGRAM}` on that worktree's PATH: {sentence}"
        );
    }
}
