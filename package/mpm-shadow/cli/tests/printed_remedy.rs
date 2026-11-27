//! What:
//! End-to-end tests of the commands `mpm-shadow` prints for a person to run,
//! in three parts,
//! each in a file of its own beside this one:
//! the sentence in the older-lock warning names a program and the verb that moves one row of the lock,
//! and that command,
//! run exactly as printed in a disposable home laid out the way this machine's is,
//! must reach mpm-shadow and move that home's machine-global lock
//! (`the_warning_for_a_moved_global_lock_reaches_the_shadow`),
//! every sentence that names `mpm` or `mpm-shadow` must name a program the scope's own install puts on `PATH`
//! (`every_sentence_names_a_program_the_install_puts_on_path`),
//! and the global sentence's own command must run as printed and clear the warning
//! (`the_global_remedy_runs_as_printed_and_clears_the_warning`).
//! The machine both latter parts build is in `a_machine_with_both_scopes_installed`.
//! This file holds what every part shares:
//! the fixture's tool id and its two versions,
//! and the two readers of printed text.
//! Why:
//! Measured on 2026-11-08:
//! the machine-wide `mpm` is the real mpm (a uv tool),
//! which answers `update -g` with `Error: No such option '-g'.`,
//! and nothing but a worktree's `PATH` ever makes `mpm` the shadow.
//! Measured on 2026-11-11 in a home laid out like this machine's:
//! the machine-global plan declares no `mpm-shadow` entry,
//! so every sentence naming `mpm-shadow` (`update -g <tool id>`,
//! `install -g`,
//! `--help`) failed with
//! `/bin/sh: line 1: mpm-shadow: command not found`,
//! exit status 127.
//! So the program the sentence names decides whether the printed command can work at all.
//! Nothing here reads or writes the real `~/.config`,
//! `~/.local` or `~/.cargo`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // printed_remedy.test.ts
//! import { text, printedCommand } from './printed_remedy/shared.ts';
//! await import('./printed_remedy/the_warning_for_a_moved_global_lock_reaches_the_shadow.ts');
//! await import('./printed_remedy/every_sentence_names_a_program_the_install_puts_on_path.ts');
//! await import('./printed_remedy/the_global_remedy_runs_as_printed_and_clears_the_warning.ts');
//! ```

/// The tests that run the older-lock warning's own command in a disposable home.
#[path = "printed_remedy/the_warning_for_a_moved_global_lock_reaches_the_shadow.rs"]
mod the_warning_for_a_moved_global_lock_reaches_the_shadow;

/// The machine both scopes install into,
/// which the two parts below build.
#[path = "printed_remedy/a_machine_with_both_scopes_installed.rs"]
mod a_machine_with_both_scopes_installed;

/// The test that every printed sentence names a program its own install puts on `PATH`.
#[path = "printed_remedy/every_sentence_names_a_program_the_install_puts_on_path.rs"]
mod every_sentence_names_a_program_the_install_puts_on_path;

/// The test that the machine-global sentence's command runs as printed and clears the warning.
#[path = "printed_remedy/the_global_remedy_runs_as_printed_and_clears_the_warning.rs"]
mod the_global_remedy_runs_as_printed_and_clears_the_warning;

/// The writer of the program files these tests start,
/// shared with the crate's other test binaries.
#[path = "written_programs/through_a_child_process.rs"]
mod through_a_child_process;

/// The tool the fixture locks:
/// a crate of this repository,
/// so the older-lock warning applies to it.
const TOOL_ID: &str = "cargo:monochromatic-demo-tool";

/// The version the fixture's global lock pins.
const LOCKED: &str = "0.0.1";

/// The version the fixture's global toolchain root holds,
/// which is newer than the pin and so raises the warning.
const INSTALLED: &str = "0.0.2";

/// The text of a stream of a finished command.
fn text(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).to_string();
}

/// The command a warning prints,
/// which the sentence holds between backticks.
fn printed_command(warning: &str) -> String {
    let Some(open) = warning.find('`') else {
        panic!("the warning prints no command: {warning}");
    };
    let rest: &str = &warning[open + 1..];
    let Some(close) = rest.find('`') else {
        panic!("the warning's command has no closing backtick: {warning}");
    };
    return rest[..close].to_string();
}
