//! What:
//!  The filling a new worktree's copied build state is given by the commit service,
//!  and the ending a control may give it.
//! Why:
//!   The add stages the worktree's copied state and hands the filling to the service,
//! which cleans the worktree's own crates out of a staging copy and publishes it (`package_repair_prewarm_job.rs`).
//! A filling whose clean is ended before it takes anything keeps nothing,
//! and at the gate of 2026-11-15T06:00-05:00 one was ended by a signal 9 from outside the tool's own runner
//! (`real_cargo_leaves_a_stale_lockfile_as_checked_out`).
//!  A control cannot produce that ending by waiting: the runner words its own ceiling and its own quiet window as
//! `CeilingReached` and `TimedOut`, and only a signal it did not send is worded `stopped by signal <n>`
//! (`package_repair_command.rs`, `run_tool_reaping_started_within`).
//! So the one ending a control needs a stand-in for is here,
//! beside the reader that says how a filling ended,
//! and it is asked for by name, so a run that does not ask for it never meets it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ending = process.env.CLI_GIT_TEST_ONLY_FILL_CLEAN; // 'killed' in the runs that end a filling's clean
//! ```

use crate::support_fixture_service::Fixture;

/// What:
///  The ending a filling's clean is given, when a control asks for one.
/// Why:
///   The runner words a tool that ended by a signal from outside its own ceiling and quiet window as
/// `stopped by signal <n>`,
/// which is how the gate's own line words the clean of the filling it could not finish,
/// and no wait of a control reaches that wording.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const A_FILLS_CLEAN_ENDING_VARIABLE = 'CLI_GIT_TEST_ONLY_FILL_CLEAN';
/// ```
pub const A_FILLS_CLEAN_ENDING_VARIABLE: &str = "CLI_GIT_TEST_ONLY_FILL_CLEAN";

/// The one ending a control may name: the clean ends itself with `SIGKILL`,
/// which the runner reports exactly as it reports a signal from outside its own bounds.
pub const ENDED_WITH_SIGKILL: &str = "killed";

/// What:
///  Give a filling's clean the ending the run asked for,
///  or leave the fixture's Cargo alone.
/// Why:
///   The naming of the ending is the environment's,
/// and the work is a named function so a control can ask for an ending without setting a variable of the whole process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (process.env.CLI_GIT_TEST_ONLY_FILL_CLEAN === 'killed') endTheFillsClean(fixture);
/// ```
pub fn end_the_fills_clean(fixture: &Fixture) {
    let asked: Option<String> = std::env::var(A_FILLS_CLEAN_ENDING_VARIABLE).ok();
    end_the_fills_clean_within(fixture, asked.as_deref());
}

/// What:
///  Replace the fixture's link to the toolchain's Cargo with a stand-in that ends with `SIGKILL` when it is the filling's
///  own clean,
///  fails the resolution that follows it,
///  and is the real Cargo for everything else.
/// Why:
///   The gate's line names two ends in one filling:
/// a clean `stopped by signal 9`,
/// and then nothing displaced and nothing kept,
/// which is the resolution after it either failing or writing no lock (`package_repair_cargo_lock.rs`).
/// The stand-in does what those lines say in the order they say it,
/// so a control meets the state the gate met and not a state it invented:
/// the fill's clean ends by signal,
/// the resolution after that signals fails as a resolution on a machine that has just lost one of its tools,
///  and the checked-out lock is never displaced.
/// Every other run of Cargo in the fixture is the real one, and the fixture's own build happens before this link is
/// replaced at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const script = `case " $* " in *' clean '*) case "$CARGO_BUILD_BUILD_DIR" in *'.monochromatic-prewarm') : > '${marker}'; kill -9 $$ ;; esac ;; esac; exec '${real}' "$@"`;
/// ```
pub fn end_the_fills_clean_within(fixture: &Fixture, ending: Option<&str>) {
    let Some(asked) = ending else { return };
    assert_eq!(
        asked, ENDED_WITH_SIGKILL,
        "the only ending a control may name for a filling's clean is {ENDED_WITH_SIGKILL}, and {A_FILLS_CLEAN_ENDING_VARIABLE} says {asked:?}"
    );
    let link: std::path::PathBuf = fixture.root.join("bin/cargo");
    let real: std::path::PathBuf = std::fs::read_link(link.as_path()).unwrap_or_else(|error| {
        panic!("the fixture's Cargo link at {} cannot be replaced, so the control asked for an ending of a filling's clean it never prepared: {error}", link.display());
    });
    let marker: std::path::PathBuf = fixture.root.join("filling-clean-ended");
    // A single quote in a path ends the quoted text, is written as itself, and starts it again.
    let quote = |path: &std::path::Path| -> String { return path.display().to_string().replace('\'', "'\\''"); };
    let script: String = format!(
        "#!/bin/sh\n\
         # The filling's own clean ends with SIGKILL, and the resolution that follows it fails: what the machine did to the\n\
         #  filling of the gate of 2026-11-15T06:00-05:00 did the same. Every other run is the real Cargo.\n\
         case \" $* \" in\n\
         \x20 *\" clean \"*)\n\
         \x20   case \"$CARGO_BUILD_BUILD_DIR\" in\n\
         \x20     *'.monochromatic-prewarm') : > '{marker}'; kill -9 $$ ;;\n\
         \x20   esac\n\
         \x20   ;;\n\
         \x20 *\" metadata \"*)\n\
         \x20   if [ -f '{marker}' ]; then printf 'error: failed to resolve the workspace: the lock file was taken from under this resolution\\n' >&2; exit 101; fi\n\
         \x20   ;;\n\
         esac\n\
         exec '{real}' \"$@\"\n",
        marker = quote(marker.as_path()),
        real = quote(real.as_path()),
    );
    std::fs::remove_file(link.as_path()).expect("the fixture's Cargo link goes before the stand-in takes its name");
    crate::support_fixture_directories::executable(link.as_path(), script.as_bytes());
}
