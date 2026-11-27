//! What:
//! The end of a bounded run:
//! the stop of the containment on every path,
//! and the manager's stop of the scope unit the run started.
//! Why:
//! Issue 6578:
//! a scope whose process was already exiting when the manager wrote its id into the scope's `cgroup.procs`
//! is counted as attached without being moved.
//! The control group then never becomes populated,
//! the kernel sends no empty notification for it,
//! and the unit stays `active (running)` with no member.
//! So the run ends its unit itself on every path it controls:
//! a normal end,
//! a bound that passes,
//! a refusal after the program started,
//! and a launcher that is gone.
//! Split out of `bounded_run_containment.rs`,
//! which keeps the stop of the containment and answers whether it was shown empty.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // bounded-run-end.ts
//! export function stopEverything(containment: Containment, child: ChildProcess): string | undefined
//! ```

/// What:
/// `use crate::bounded_run::{Bound, RunError, ScopeState};` imports the bound of a run,
/// its error and the state of its journal.
/// Why:
/// The path that fails a run after its program started takes all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Bound, RunError, ScopeState } from 'crate/bounded_run';
/// ```
use crate::bounded_run::{
    Bound,
    RunError,
    ScopeState,
};
/// What:
/// `use crate::bounded_run_containment::{Containment, lose, settle, stop_containment};` imports the containment of a run,
/// the mark that writers may remain,
/// the record that a run ended and the stop itself.
/// Why:
/// The end asks the containment to stop,
/// keeps that answer,
/// adds the end of the unit after it,
/// and settles or marks the journal the same way the containment does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Containment, lose, settle, stopContainment } from 'crate/bounded_run_containment';
/// ```
use crate::bounded_run_containment::{
    Containment,
    lose,
    settle,
    stop_containment,
};
/// What:
/// `use crate::control_group::ControlGroup;` imports the scope whose unit is stopped.
/// Why:
/// The unit name and the scope's directory belong to that type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ControlGroup } from 'crate/control_group';
/// ```
use crate::control_group::ControlGroup;
/// What:
/// `use crate::control_group::retire::{RETIRE_BOUND, retire_unit};` imports the stop of a unit and its bound.
/// Why:
/// The one way a unit is stopped is shared with the controls that plant such a unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { RETIRE_BOUND, retireUnit } from 'crate/control_group/retire';
/// ```
use crate::control_group::retire::{
    RETIRE_BOUND,
    retire_unit,
};
/// What:
/// `use crate::control_group_route_discovery::Route;` imports the route a run's scope was planned on.
/// Why:
/// The route holds the program that asks the manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Route } from 'crate/control_group_route_discovery';
/// ```
use crate::control_group_route_discovery::Route;
/// What:
/// `use std::process::Child;` imports the handle of a started child.
/// Why:
/// The stop acts on the direct child the run started.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ChildProcess } from 'node:child_process';
/// ```
use std::process::Child;
/// What:
/// `use std::rc::Rc;` imports the reference-counted pointer type.
/// Its siblings are `Box` (which owns its value) and `Arc` (which counts atomically for several threads).
/// Why:
/// The failed-start path lends the journal state to the settle and lose calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // every object reference in TS is shared and counted by the garbage collector
/// ```
use std::rc::Rc;

/// What:
/// `fn retire(group: &ControlGroup, route: &Route)` asks the manager to stop the scope unit of a run that has ended,
/// and warns when the manager may still load it.
/// Why:
/// The scope is already shown empty,
/// so this is housekeeping for the unit and never a claim about a writer.
/// A manager that cannot be asked does not turn the run into "not quiescent",
/// but a person has to be told that the unit may stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function retire(group: ControlGroup, route: Route): void
/// ```
fn retire(
    group: &ControlGroup,
    route: &Route,
) {
    let unit: String = group.unit();
    // What:     `if let Err(reason) = retire_unit(...)` runs the warning only when the retire answered a failure.
    //           `&route.systemctl` and `&unit` lend the two names to the call.
    // Why:      A retired unit needs nothing said,
    //           and a warning on every run would be noise.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const reason = retireUnit(route.systemctl, unit, RETIRE_BOUND).error
    // if (reason) console.error(`... ${unit} ... ${reason}`)
    // ```
    if let Err(reason) = retire_unit(&route.systemctl, &unit, RETIRE_BOUND) {
        eprintln!(
            "cargo-shadow: warning: the scope unit {unit} of the run that just ended may still be loaded ({reason})"
        );
    }
}

/// What:
/// `fn stop_everything(containment, child) -> Result<(), String>` ends the run early or at its normal exit:
/// it stops what the containment holds,
/// asks the manager to stop the run's scope unit,
/// and answers `Ok` only if all of the containment is shown gone,
/// or the words a refusal carries saying what was not shown and why.
/// Why:
/// This is the one way a run is ended,
/// on timeout,
/// on a normal exit and on every error path.
/// The unit is asked to stop on every one of those paths,
/// because a scope the manager never noticed becoming empty would otherwise stay loaded.
/// The direct child goes first:
/// it is the process that becomes the program,
/// so once it is reaped nothing of this program's can
/// still start the program in a scope that is not there yet.
/// A scope that cannot be stopped,
/// and any failure to inspect or stop it,
/// is "not shown gone",
/// and the text says how long the stop waited and what it was waiting for (issue 6528).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function stopEverything(containment: Containment, child: ChildProcess): string | undefined
/// ```
pub(crate) fn stop_everything(
    containment: &Containment,
    child: &mut Child,
) -> Result<(), String> {
    // What:     `let outcome: Result<(), String> = stop_containment(...)` keeps the proof (or the shortfall) while the unit is ended.
    //           `Result<(), String>` is the success-or-reason pair of the whole stop.
    // Why:      The unit has to be ended whatever the proof says,
    //           and the caller must still receive the proof itself.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = stopContainment(containment, child)
    // ```
    let outcome: Result<(), String> = stop_containment(containment, child);
    // What:     `if let Containment::Scope { group, route } = containment` runs the retirement only for a delegated scope.
    //           The process-group path has no unit,
    //           and `group` and `route` are borrowed views into the containment.
    // Why:      Only a scope leaves a unit behind;
    //           the weaker path leaves nothing in the manager.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (containment.kind === 'scope') retire(containment.group, containment.route)
    // ```
    if let Containment::Scope {
        group,
        route,
    } = containment
    {
        // What:     `if outcome.is_ok()` asks whether the stop showed the scope empty before the unit is stopped.
        // Why:      A stop that could not show the scope empty may still have a member that can write,
        //           and the refusal has to be inspectable (its report names the members);
        //           asking the manager to stop the unit would kill that member behind the report.
        //           Such a unit is not the loaded-and-empty leftover this step exists for.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (outcome.ok) retire(group, route)
        // ```
        if outcome.is_ok() {
            retire(group, route);
        }
    }
    return outcome;
}

/// The controls that every path a run controls retires its scope unit (issue 6578).
#[cfg(test)]
#[path = "bounded_run_retire_tests.rs"]
mod retire_tests;

/// What:
/// `fn fail_started(containment, child, scope, bound, error) -> RunError` ends a run that failed after its program started.
/// Why:
/// Any failure on the containment path after the start is "not quiescent" unless everything is shown gone (contract A2):
/// it must not become an ordinary failure that lets the caller roll back or accept under a writer that may remain (7a F3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failStarted(containment: Containment, child: ChildProcess, scope: ScopeState | undefined, bound: Bound, error: RunError): RunError
/// ```
pub(crate) fn fail_started(
    containment: &Containment,
    child: &mut Child,
    scope: &Option<Rc<ScopeState>>,
    bound: &Bound,
    error: RunError,
) -> RunError {
    return match stop_everything(containment, child) {
        Ok(()) => {
            settle(containment, scope);
            error
        }
        Err(shortfall) => {
            lose(scope, shortfall.clone());
            RunError::NotQuiescent {
                what: bound.what.clone(),
                shortfall,
            }
        }
    };
}
