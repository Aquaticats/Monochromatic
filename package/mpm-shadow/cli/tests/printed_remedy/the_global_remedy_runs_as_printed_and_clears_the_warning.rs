//! What:
//! The test that runs the machine-global sentence's command as printed:
//! a fresh shell starts it with only the scope's own directories on `PATH`,
//! the command must exit 0,
//! and the lock it wrote must name the version that runs,
//! with the request that asks for it,
//! after which the demo tool runs with no older-lock warning.
//! Why:
//! The sentence is the whole remedy a person is given,
//! so it has to be runnable exactly as printed in a shell that has never seen the worktree:
//! a sentence naming a program that shell cannot find leaves the lock where it was.
//! Measured on 2026-11-08:
//! `update -g` typed to this machine's `mpm` answered `Error: No such option '-g'.`,
//! because that `mpm` is the real one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // printed_remedy/the_global_remedy_runs_as_printed_and_clears_the_warning.test.ts
//! const remedy = printedCommand(warning);
//! expect(shell(remedy, Place.Global).status).toBe(0);
//! expect(parseLock(globalLock()).version).toBe(INSTALLED);
//! ```

use std::fs;
use std::path::PathBuf;

use mpm_shadow::toolchain::lock::parse_lock;

use crate::a_machine_with_both_scopes_installed::{
    DEMO_BIN,
    Machine,
    Place,
    SHADOW_PROGRAM,
};
use crate::{
    INSTALLED,
    LOCKED,
    TOOL_ID,
    printed_command,
    text,
};

#[test]
fn the_global_remedy_runs_as_printed_and_clears_the_warning() {
    let machine = Machine::new();
    let installed = machine.install_global();
    assert_eq!(
        installed.status.code(),
        Some(0),
        "the machine-global install failed:\n{}",
        text(&installed.stderr)
    );
    let program: PathBuf = machine.farm().join(SHADOW_PROGRAM);
    assert!(
        program.exists(),
        "the install puts the program the printed command starts in the farm a fresh shell reads: {}",
        program.display()
    );

    // The plan now asks for the version that runs, which is the state the warning's own command repairs.
    machine.write_global_plan(INSTALLED);
    let before = machine.shell(DEMO_BIN, Place::Global);
    assert_eq!(
        before.status.code(),
        Some(0),
        "the demo tool runs:\n{}",
        text(&before.stderr)
    );
    let warning: String = text(&before.stderr);
    assert!(
        warning.contains(TOOL_ID) && warning.contains(LOCKED) && warning.contains(INSTALLED),
        "{warning}"
    );
    let remedy: String = printed_command(&warning);
    assert_eq!(
        remedy,
        format!("{SHADOW_PROGRAM} update -g {TOOL_ID}"),
        "the sentence names the shadow and the verb that moves one row"
    );

    let ran = machine.shell(&remedy, Place::Global);
    assert_eq!(
        ran.status.code(),
        Some(0),
        "`{remedy}` must run as printed in a fresh shell; it printed\n{}\n{}",
        text(&ran.stdout),
        text(&ran.stderr)
    );

    let lock: String =
        fs::read_to_string(machine.global_lock_file()).expect("read the global lock");
    let parsed = parse_lock(&lock, "the lock the remedy wrote").expect("the lock parses");
    let tool = parsed
        .lock
        .tool(TOOL_ID)
        .expect("the lock still holds the tool");
    assert_eq!(
        tool.version, INSTALLED,
        "the lock now names the version that runs: {lock}"
    );
    assert_eq!(
        tool.declaration.request.range, INSTALLED,
        "and the request that asks for it: {lock}"
    );

    let after = machine.shell(DEMO_BIN, Place::Global);
    assert_eq!(
        after.status.code(),
        Some(0),
        "the demo tool still runs:\n{}",
        text(&after.stderr)
    );
    assert_eq!(
        text(&after.stderr),
        "",
        "the older-lock warning is gone, because the lock names the version that runs"
    );
}
