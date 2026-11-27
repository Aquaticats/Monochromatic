//! What:
//! The step that ends a run's scope unit after the run has shown its control group empty:
//! a stop request to the manager and the reading of the manager's answer.
//! Why:
//! Issue 6578:
//! a transient scope whose process was already exiting when the manager wrote its process id into
//! the scope's `cgroup.procs` is counted as attached without being moved
//! (`cg_attach` returns success for a task that is leaving).
//! The scope's control group therefore never becomes populated,
//! the kernel sends no empty notification for it,
//! and the unit stays `active (running)` with `populated 0` and no member.
//! A run cannot make the manager re-read that state,
//! but it can ask the manager to stop the unit it started:
//! the stop job ends the unit, and `CollectMode=inactive-or-failed` (the `--collect` of the launcher)
//! then frees it.
//! Split out of `bounded_run_containment.rs`,
//! which decides how a run is contained and stops it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-retire.ts
//! export function retireUnit(systemctl: string, unit: string, boundMs: number): void  // throws the reason
//! ```

/// What:
/// `use crate::control_group_route_child_runs::run_plain;` imports the short program runner of the route discovery.
/// Why:
/// Asking the manager to stop a unit is the same kind of bounded question as asking it where the slice is,
/// and that question already has one implementation with a deadline around the reading, the killing and the reaping.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { runPlain } from 'crate/control_group_route_child_runs';
/// ```
use crate::control_group_route_child_runs::run_plain;
/// What:
/// `use std::time::Duration;` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The stop request and the question that follows it each get a deadline,
/// and this file carries the bound both use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// How long the manager is given to answer a stop request and to say whether the unit is still loaded.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// measurement beside it:
/// on 2026-11-27 a `systemctl --user stop` of an empty `cargo-shadow-*.scope` answered in 4 ms to 5 ms
/// (three runs, this host, systemd 262.2-1-arch),
/// so five seconds only ends a manager that does not answer.
pub(crate) const RETIRE_BOUND: Duration = Duration::from_secs(5);

/// What:
/// `pub(crate) fn retire_unit(systemctl: &str, unit: &str, bound: Duration) -> Result<(), String>` asks the manager to stop
/// `unit`,
/// and answers `Ok` when the unit is gone afterwards or was already gone,
/// or the reason it may still be loaded.
/// Why:
/// The run's scope is already shown empty,
/// so this is housekeeping and never a claim about a writer;
/// the caller reports a failure as a warning and does not turn it into "not quiescent".
/// A unit the manager collected before the request is not a failure:
/// its stop request is refused as "not loaded", and the question afterwards (`LoadState`) says `not-found`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function retireUnit(systemctl: string, unit: string, bound: number): void  // throws the reason
/// ```
pub(crate) fn retire_unit(
    systemctl: &str,
    unit: &str,
    bound: Duration,
) -> Result<(), String> {
    // What:     `run_plain(...)` starts `systemctl --user --no-ask-password stop <unit>` under `bound` and returns its standard
    //           output, or the reason it failed. The flags are the ones the machine's rules require of every `systemctl` call:
    //           `--user` keeps it on this account's manager, and `--no-ask-password` never opens a password dialog on a desktop.
    // Why:      The unit this run made has to be ended by the manager that made it; the request is a task of its own, under its own
    //           deadline.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stopped = runPlain(systemctl, ['--user', '--no-ask-password', 'stop', unit], [], bound)
    // ```
    let stopped: Result<String, String> = run_plain(
        systemctl,
        &["--user", "--no-ask-password", "stop", unit],
        &[],
        bound,
    );
    // What:     `match stopped { Ok(_) => ..., Err(reason) => ... }` asks which variant the stop request answered. `Ok` means the
    //           manager completed the stop job; the arm returns `Ok(())` at once, because `--collect` makes the manager free a
    //           scope that is inactive or failed without another request.
    // Why:      A successful stop is the whole answer; there is nothing left to ask.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (stopped.ok) return
    // const why = stopped.error
    // ```
    let why: String = match stopped {
        Ok(_) => return Ok(()),
        Err(reason) => reason,
    };
    // What:     The second question asks the manager what `LoadState` the unit has now, so a stop refused because the unit was
    //           already collected (`Unit ... not loaded.`) is told apart from a manager that refused for another reason.
    // Why:      Reading the state is exact and needs no message text to be parsed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const loaded = runPlain(systemctl, ['--user', '--no-ask-password', 'show', unit, '--property=LoadState', '--value'], [], bound)
    // ```
    let loaded: Result<String, String> = run_plain(
        systemctl,
        &[
            "--user",
            "--no-ask-password",
            "show",
            unit,
            "--property=LoadState",
            "--value",
        ],
        &[],
        bound,
    );
    // What:     `match loaded { ... }` reads the second answer: `Ok(answer)` carries the state text (its `&str` view is `&str`, the
    //           sibling of an owned `String`), and `Err(reason)` carries why the question could not be answered. The guard
    //           `if answer.trim() == "not-found"` treats the manager's "this unit is not loaded" as success. `format!` builds each
    //           failure sentence, and the tail `match` returns it.
    // Why:      A collected unit is gone, and every other answer must reach the caller with both the stop's reason and what the
    //           manager still shows, so the warning a person reads names the unit and the state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const answer = loaded.value
    // if (answer.trim() === 'not-found') return
    // throw new Error(`stop failed (${why}) and the manager still loads it as ${answer.trim()}`)
    // ```
    return match loaded {
        Ok(answer) if answer.trim() == "not-found" => Ok(()),
        Ok(answer) => Err(format!(
            "`{systemctl} --user stop {unit}` failed ({why}) and the manager still loads the unit as {}",
            answer.trim()
        )),
        Err(reason) => Err(format!(
            "`{systemctl} --user stop {unit}` failed ({why}) and whether the unit is still loaded could not be asked ({reason})"
        )),
    };
}

/// The controls of this module (issue 6578).
#[cfg(test)]
#[path = "control_group_retire_tests.rs"]
mod tests;
